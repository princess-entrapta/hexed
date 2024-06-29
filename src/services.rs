use std::{
    collections::{HashMap, HashSet},
    hash::RandomState,
    str::FromStr,
};

use crate::{
    abilities::{Ability, AbilityName, TargetType},
    schemas::{
        AbilityTargets, ActionLog, ActionLogResponse, Coords, DeployEntitiesRequest, Entity,
        Gamestate, ScenarioPlayer, Tile, TileType,
    },
    stores::{
        database::{Game, Repository},
        events::Events,
    },
};

#[derive(Debug)]
pub enum ServiceError {
    DbError(sqlx::error::Error),
    BadRequest(String),
    QueueError(String),
    NotFound,
    Unauthorized,
}

impl From<sqlx::error::Error> for ServiceError {
    fn from(value: sqlx::error::Error) -> Self {
        return Self::DbError(value);
    }
}

impl From<std::sync::mpsc::SendError<Gamestate>> for ServiceError {
    fn from(value: std::sync::mpsc::SendError<Gamestate>) -> Self {
        return Self::QueueError(value.to_string());
    }
}

impl ToString for ServiceError {
    fn to_string(&self) -> String {
        match self {
            Self::DbError(err) => err.to_string(),
            Self::BadRequest(err) | Self::QueueError(err) => err.to_owned(),
            Self::NotFound => "Not found".to_string(),
            Self::Unauthorized => "Unauthorized".to_string(),
        }
    }
}

pub async fn get_gamestate(repo: Repository, game_id: i64) -> Result<Gamestate, ServiceError> {
    let mut to_play = repo.get_trait_entity(&game_id).await?;
    let entities = repo.get_game_entities(&game_id).await?;
    let blocking_entities = entities
        .clone()
        .into_iter()
        .map(|e_with_ressource| e_with_ressource.entity)
        .filter(|e| e.scenario_player_id != to_play.scenario_player_id)
        .collect();
    let allied_entities = entities
        .clone()
        .into_iter()
        .map(|e_with_resource| e_with_resource.entity)
        .filter(|e| e.scenario_player_id == to_play.scenario_player_id)
        .collect::<Vec<_>>();

    let map: Vec<Tile> = repo.get_game_tiles(&game_id).await?;
    let (los_tiles, allied_vision) = get_los_map(
        to_play.clone(),
        &allied_entities,
        map.clone(),
        &blocking_entities,
    )
    .await;
    let los_coords: HashSet<(i64, i64), RandomState> =
        HashSet::from_iter(los_tiles.clone().into_iter().map(|t| (t.x, t.y)));
    let visible_entities = Vec::from_iter(
        entities
            .clone()
            .into_iter()
            .filter(|e| {
                los_coords.contains(&(e.entity.x, e.entity.y))
                    || e.entity.scenario_player_id == to_play.scenario_player_id
            })
            .map(|e| e),
    );
    let visible_entities_id =
        HashSet::from_iter(visible_entities.clone().into_iter().map(|e| e.entity.id));
    let logs =
        get_logs_from_los(repo, game_id, to_play.last_move_time, visible_entities_id).await?;
    Ok(Gamestate {
        id: game_id,
        playing: to_play.id,
        entities: visible_entities.clone(),
        abilities: to_play
            .get_class()
            .get_ability_list()
            .into_iter()
            .map(|name| AbilityTargets {
                name: name.clone(),
                costs: Ability {
                    name: name.clone(),
                    caster: &mut to_play,
                }
                .get_costs(),
                targets: los_tiles
                    .clone()
                    .into_iter()
                    .filter(|tile| {
                        let target_entity = visible_entities
                            .clone()
                            .into_iter()
                            .filter(|e| e.entity.x == tile.x && e.entity.y == tile.y)
                            .map(|e| e.entity.clone())
                            .collect::<Vec<Entity>>()
                            .get(0)
                            .cloned();
                        {
                            is_valid_target(
                                &Ability {
                                    name: name.clone(),
                                    caster: &mut to_play,
                                },
                                &tile,
                                &target_entity,
                                &blocking_entities,
                                &map,
                            )
                            .is_ok()
                        }
                    })
                    .map(|t| Coords { x: t.x, y: t.y })
                    .collect(),
            })
            .collect(),
        visible_tiles: los_tiles,
        allied_vision,
        logs,
    })
}

pub fn get_distance(x1: i64, y1: i64, x2: i64, y2: i64) -> f64 {
    f64::sqrt((((x1 - x2) * (x1 - x2)) as f64 / 4.0) + (((y1 - y2) * (y1 - y2)) as f64 * 3.0 / 4.0))
}

pub async fn transfer_entity(
    repo: Repository,
    game_id: i64,
    user_from: String,
    user_to: String,
) -> Result<(), ServiceError> {
    let entity = repo.get_trait_entity(&game_id).await?;
    if entity.user_id != user_from {
        return Err(ServiceError::Unauthorized);
    }
    repo.transfer_entity(&game_id, &entity.id, &user_to).await?;
    Ok(())
}

pub async fn deploy_entities(
    repo: Repository,
    events: Events,
    user_id: String,
    game_id: i64,
    entites: DeployEntitiesRequest,
) -> Result<(), ServiceError> {
    // Check if game is Open
    // Check if scenario_player_id already deployed
    // Check points
    // Check location
    let seat_id = repo
        .add_seat(&user_id, &game_id, &entites.scenario_player_id)
        .await?;
    for ent in entites.entities {
        let entity = repo.add_entity_for_player(&ent, &game_id, &seat_id).await?;
        for resource in ent.get_class().get_resource_list().into_iter() {
            repo.add_resource(&entity, &game_id, &resource).await?;
        }
    }

    if repo.count_seats_for_game(&game_id).await?
        == repo.count_scenario_players_for_game(&game_id).await?
    {
        repo.start_game(&game_id).await?;
        let _res = tick(events, repo, game_id).await;
    };
    Ok(())
}

pub async fn tick(events: Events, repo: Repository, game_id: i64) -> Result<(), ServiceError> {
    let entity_to_play = repo.get_trait_entity(&game_id).await?;
    tracing::info!("sending tick to {}", entity_to_play.user_id);
    events
        .send_event(get_gamestate(repo, game_id).await?, entity_to_play.user_id)
        .await
}

pub fn get_possible_moves(start_x: i64, start_y: i64, end_x: i64, end_y: i64) -> Vec<(i64, i64)> {
    let diff_x = end_x - start_x;
    let diff_y = end_y - start_y;
    if (diff_x, diff_y) == (0, 0) {
        return Vec::new();
    }
    if diff_y == 0 {
        if diff_x < 0 {
            return vec![(-2, 0)];
        }
        return vec![(2, 0)];
    }
    if diff_y > 0 {
        if diff_x > 0 {
            if diff_y > diff_x {
                return vec![(-1, 1), (1, 1)];
            } else if diff_y < diff_x {
                return vec![(2, 0), (1, 1)];
            } else {
                return vec![(1, 1)];
            }
        } else {
            if diff_y > -diff_x {
                return vec![(-1, 1), (1, 1)];
            } else if diff_y < -diff_x {
                return vec![(-2, 0), (-1, 1)];
            } else {
                return vec![(-1, 1)];
            }
        }
    } else {
        if diff_x > 0 {
            if -diff_y > diff_x {
                return vec![(-1, -1), (1, -1)];
            } else if -diff_y < diff_x {
                return vec![(2, 0), (1, -1)];
            } else {
                return vec![(1, -1)];
            }
        } else {
            if -diff_y > -diff_x {
                return vec![(-1, -1), (1, -1)];
            } else if -diff_y < -diff_x {
                return vec![(-2, 0), (-1, -1)];
            } else {
                return vec![(-1, -1)];
            }
        }
    }
}

pub fn get_los_line(start_x: i64, start_y: i64, end_x: i64, end_y: i64) -> HashSet<(i64, i64)> {
    let mut init_pos = HashSet::new();
    init_pos.insert((start_x, start_y));
    if start_x == end_x && start_y == end_y {
        return init_pos;
    }
    let max_dist = get_distance(start_x, start_y, end_x, end_y) + 1.001;
    let moves = get_possible_moves(start_x, start_y, end_x, end_y);
    let mut reached_end = false;

    loop {
        let mut new_pos = HashSet::new();
        for tile in init_pos.clone() {
            for mv in moves.clone() {
                let new_tile = (tile.0 + mv.0, tile.1 + mv.1);
                if get_distance(new_tile.0, new_tile.1, start_x, start_y)
                    + get_distance(new_tile.0, new_tile.1, end_x, end_y)
                    > max_dist
                {
                    continue;
                }
                if new_tile == (end_x, end_y) {
                    reached_end = true;
                    break;
                }
                new_pos.insert(new_tile);
            }
        }
        init_pos.extend(new_pos);
        if reached_end {
            return init_pos;
        }
    }
}

pub fn has_los(
    start_x: i64,
    start_y: i64,
    end_x: i64,
    end_y: i64,
    map_by_coords: &HashMap<(i64, i64), Tile, RandomState>,
    blocking_entities: &Vec<Entity>,
    is_blocking: fn(&TileType) -> bool,
) -> Result<(), Tile> {
    let dist_start_end = get_distance(start_x, start_y, end_x, end_y);
    if dist_start_end < 0.01 {
        return Ok(());
    }
    let mut coords: Vec<(i64, i64)> = get_los_line(start_x, start_y, end_x, end_y)
        .into_iter()
        .collect();
    let blocked_coords: HashSet<(i64, i64), RandomState> =
        HashSet::from_iter(blocking_entities.into_iter().map(|e| (e.x, e.y)));
    coords.sort_by(|a, b| {
        get_distance(start_x, start_y, a.0, a.1)
            .partial_cmp(&get_distance(start_x, start_y, b.0, b.1))
            .unwrap()
    });
    for (x, y) in coords {
        let distance_to_line =
            ((((end_x - start_x) as f64 / 2.0) * (y - start_y) as f64 * (0.75_f64).sqrt())
                - ((x - start_x) as f64 / 2.0) * ((end_y - start_y) as f64 * (0.75_f64).sqrt()))
            .abs();
        if distance_to_line > dist_start_end / 2.0 {
            continue;
        }
        let is_blocked: Option<&(i64, i64)> = blocked_coords.get(&(x, y));
        if is_blocked.is_some() {
            return Err(map_by_coords.get(&(x, y)).unwrap().clone());
        }
        let try_cur_tile = map_by_coords.get(&(x, y));
        if try_cur_tile.is_none() {
            continue;
        }
        let cur_tile = try_cur_tile.unwrap();
        if is_blocking(&cur_tile.tile_type) {
            return Err(map_by_coords.get(&(x, y)).unwrap().clone());
        }
    }
    Ok(())
}

pub async fn get_los_map(
    entity: Entity,
    allied_entities: &Vec<Entity>,
    map: Vec<Tile>,
    blocking_entities: &Vec<Entity>,
) -> (HashSet<Tile>, HashSet<Tile>) {
    let map_by_coords =
        HashMap::from_iter(map.clone().into_iter().map(|t| ((t.x, t.y), t.clone())));
    let mut allied_vision = HashSet::new();
    let mut los_map = HashSet::new();
    for tile in map.clone() {
        let los_result = has_los(
            entity.x,
            entity.y,
            tile.x,
            tile.y,
            &map_by_coords,
            blocking_entities,
            TileType::is_blocking_sight,
        );
        los_map.insert(match los_result {
            Ok(_) => tile,
            Err(wall) => {
                for ally in allied_entities.into_iter() {
                    let ally_tile = match has_los(
                        ally.x,
                        ally.y,
                        tile.x,
                        tile.y,
                        &map_by_coords,
                        blocking_entities,
                        TileType::is_blocking_sight,
                    ) {
                        Ok(_) => tile.clone(),
                        Err(ally_wall) => ally_wall,
                    };
                    allied_vision.insert(ally_tile);
                }
                wall
            }
        });
    }
    (
        los_map.clone(),
        allied_vision.difference(&los_map).cloned().collect(),
    )
}

pub async fn get_logs_from_los(
    repo: Repository,
    game_id: i64,
    turn_time: i64,
    visible_entities: HashSet<i64>,
) -> Result<Vec<ActionLogResponse>, ServiceError> {
    let logs = repo.get_logs_since(&game_id, &turn_time).await?;
    Ok(logs
        .into_iter()
        .filter(|log| {
            visible_entities.contains(&log.caster)
                || log
                    .target_entity
                    .is_some_and(|e| visible_entities.contains(&e))
        })
        .map(|log| ActionLogResponse {
            action_name: AbilityName::from_str(log.action_name.as_str()).unwrap(),
            turn_time: log.turn_time,
            caster: visible_entities.get(&log.caster).cloned(),
            target_entity: log
                .target_entity
                .map(|e| visible_entities.get(&e).cloned())
                .unwrap_or(None),
        })
        .collect())
}

pub async fn get_active_game(repo: Repository, user_id: String) -> Result<Vec<Game>, ServiceError> {
    let mut games = repo.running_games(&user_id).await?;
    games.extend(repo.open_games().await?.into_iter());
    Ok(games)
}

pub async fn get_scenarios(repo: Repository) -> Result<Vec<i64>, ServiceError> {
    Ok(repo.get_scenarios().await?)
}

pub async fn get_scenario_players(
    repo: Repository,
    scenario_id: i64,
) -> Result<Vec<ScenarioPlayer>, ServiceError> {
    Ok(repo.get_scenario_players(&scenario_id).await?)
}

pub async fn get_available_scenario_players(
    repo: Repository,
    game_id: i64,
) -> Result<Vec<ScenarioPlayer>, ServiceError> {
    Ok(repo.get_available_scenario_players(&game_id).await?)
}

pub async fn new_game(
    repo: Repository,
    user_id: String,
    scenario_id: i64,
) -> Result<i64, ServiceError> {
    Ok(repo.new_game(&user_id, &scenario_id).await?)
}

pub fn is_valid_target(
    ability: &Ability,
    target: &Tile,
    target_entity: &Option<Entity>,
    blocking_entities: &Vec<Entity>,
    map: &Vec<Tile>,
) -> Result<(), ServiceError> {
    let distance = get_distance(ability.caster.x, ability.caster.y, target.x, target.y);
    if distance > ability.max_range(ability.caster.get_class()) {
        return Err(ServiceError::BadRequest("Out of range".to_string()));
    }
    match ability.target_type() {
        TargetType::Walkable => {
            if target_entity.is_some() {
                return Err(ServiceError::BadRequest(
                    "Target must be walkable".to_string(),
                ));
            }
            match target.tile_type {
                TileType::Wall | TileType::DeepWater => {
                    return Err(ServiceError::BadRequest(
                        "Can't walk on that tile".to_string(),
                    ))
                }
                _ => {}
            }
        }
        TargetType::Ennemy => {
            if target_entity.is_none()
                || target_entity
                    .clone()
                    .is_some_and(|e| e.scenario_player_id == ability.caster.scenario_player_id)
            {
                return Err(ServiceError::BadRequest(
                    "Target must be an ennemy".to_string(),
                ));
            }
        }
        TargetType::Ally => {
            if target_entity.is_none()
                || target_entity
                    .clone()
                    .is_some_and(|e| e.scenario_player_id != ability.caster.scenario_player_id)
            {
                return Err(ServiceError::BadRequest(
                    "Target must be an ally".to_string(),
                ));
            }
        }
        TargetType::Selfcast => {
            if target_entity != &Some(ability.caster.clone()) {
                return Err(ServiceError::BadRequest(
                    "This ability is self-cast".to_string(),
                ));
            }
        }
    }
    if ability.needs_los() {
        let map_by_coords =
            HashMap::from_iter(map.clone().into_iter().map(|t| ((t.x, t.y), t.clone())));
        if has_los(
            ability.caster.x,
            ability.caster.y,
            target.x,
            target.y,
            &map_by_coords,
            &blocking_entities,
            TileType::is_blocking_sight,
        )
        .is_err()
        {
            return Err(ServiceError::BadRequest(
                "Target is out of line of sight".to_string(),
            ));
        }
    }
    Ok(())
}

pub async fn use_ability(
    repo: Repository,
    events: Events,
    game_id: i64,
    user_id: String,
    ability_name: AbilityName,
    target: Coords,
) -> Result<(), ServiceError> {
    let mut entity = repo.get_trait_entity(&game_id).await?;
    if entity.user_id != user_id {
        return Err(ServiceError::Unauthorized);
    }
    let entity_id = entity.id;
    let target_entity = repo.get_entity_at(&game_id, &target.x, &target.y).await?;
    let mut ability = Ability {
        name: ability_name.clone(),
        caster: &mut entity,
    };

    let game_entities = repo.get_game_entities(&game_id).await?;
    let blocking_entities: Vec<Entity> = game_entities
        .clone()
        .into_iter()
        .map(|e_with_ressource| e_with_ressource.entity)
        .filter(|e| e.scenario_player_id != ability.caster.scenario_player_id)
        .collect();
    let map = repo.get_game_tiles(&game_id).await?;
    let target_tile = repo.get_tile_at(&game_id, &target.x, &target.y).await?;
    is_valid_target(
        &ability,
        &target_tile,
        &target_entity,
        &blocking_entities,
        &map,
    )?;
    let costs = ability.get_costs();
    for (resource_name, cost) in costs.clone() {
        let resource = repo
            .get_resource(&game_id, &entity_id, resource_name.as_str())
            .await?;
        if resource.resource_current < cost {
            return Err(ServiceError::BadRequest(format!(
                "Ability is not ready, lack resource {}",
                resource_name
            )));
        }
    }
    let delay = ability.get_delay(&repo, &game_id, target.clone()).await;
    ability
        .apply(&repo, game_id, target.clone(), target_entity.clone())
        .await?;
    for (resource_name, cost) in costs {
        let mut resource = repo
            .get_resource(&game_id, &entity_id, resource_name.as_str())
            .await?;
        resource.resource_current -= cost;
        repo.set_resource(&resource).await?;
    }
    ability.caster.last_move_time = ability.caster.next_move_time;
    ability.caster.next_move_time += delay;
    tracing::info!("{:?}", ability.caster);

    let new_entity: Entity = repo.get_trait_entity(&game_id).await?;
    let elapsed_time = new_entity.next_move_time - ability.caster.last_move_time;

    repo.log_action(&ActionLog {
        game_id,
        turn_time: ability.caster.last_move_time,
        caster: entity_id,
        target_entity: target_entity.map(|e| e.id),
        action_name: ability_name.to_string(),
    })
    .await?;
    repo.set_entity(&game_id, &ability.caster).await?;
    repo.increment_resources(&game_id, &elapsed_time).await?;
    let _res = events
        .send_event(get_gamestate(repo, game_id).await?, new_entity.user_id)
        .await;

    Ok(())
}

#[cfg(test)]
mod test {
    use std::{
        collections::{HashMap, HashSet},
        hash::RandomState,
    };

    use sqlx::Sqlite;

    use crate::{
        abilities::AbilityName,
        schemas::{
            AbilityTargets, Coords, Entity, EntityWithResources, Gamestate, Resource, Tile,
            TileType,
        },
        services::{get_los_line, get_possible_moves, has_los},
        stores::{database::Repository, events::Events},
    };

    use super::get_distance;

    fn make_map(vec: Vec<Tile>) -> HashMap<(i64, i64), Tile, RandomState> {
        HashMap::from_iter(vec.into_iter().map(|t| ((t.x, t.y), t.clone())))
    }

    #[test]
    fn test_get_distance() {
        assert!(get_distance(0, 0, 2, 0) < 1.001);
        assert!(get_distance(0, 0, 2, 0) > 0.999);
    }

    #[test]
    fn test_get_distance_y() {
        assert!(get_distance(0, 0, 1, 1) < 1.001);
        assert!(get_distance(0, 0, 1, 1) > 0.999);
    }

    #[test]
    fn test_get_distance_neg() {
        assert!(get_distance(0, 0, 1, -1) < 1.001);
        assert!(get_distance(0, 0, 1, -1) > 0.999);
    }

    #[test]
    fn test_get_distance_diag() {
        assert!(get_distance(0, 0, 0, 2) < 1.8);
        assert!(get_distance(0, 0, 0, 2) > 1.7);
    }

    #[test]
    fn test_get_possible_moves() {
        assert_eq!(get_possible_moves(0, 0, 2, 2), vec![(1, 1)])
    }
    #[test]
    fn test_get_possible_moves_2() {
        assert_eq!(get_possible_moves(0, 0, -2, -2), vec![(-1, -1)])
    }
    #[test]
    fn test_get_possible_moves_3() {
        assert_eq!(get_possible_moves(0, 0, 1, 3), vec![(-1, 1), (1, 1)])
    }
    #[test]
    fn test_get_possible_moves_4() {
        assert_eq!(get_possible_moves(0, 0, 3, 1), vec![(2, 0), (1, 1)])
    }
    #[test]
    fn test_get_possible_moves_5() {
        assert_eq!(get_possible_moves(0, 0, -3, 1), vec![(-2, 0), (-1, 1)])
    }
    #[test]
    fn test_get_possible_moves_6() {
        assert_eq!(get_possible_moves(0, 0, -3, -1), vec![(-2, 0), (-1, -1)])
    }
    #[test]
    fn test_los_cone() {
        assert_eq!(
            get_los_line(0, 0, 3, 1),
            HashSet::from_iter(vec![(0, 0), (2, 0), (1, 1)])
        )
    }

    #[test]
    fn test_has_los() {
        assert_eq!(
            has_los(
                -2,
                0,
                2,
                0,
                &make_map(vec![Tile {
                    x: 0,
                    y: 0,
                    tile_type: crate::schemas::TileType::Wall
                }]),
                &vec![],
                TileType::is_blocking_sight
            ),
            Err(Tile {
                x: 0,
                y: 0,
                tile_type: crate::schemas::TileType::Wall
            })
        )
    }
    #[test]
    fn test_has_los_through_floor() {
        assert_eq!(
            has_los(
                -2,
                0,
                2,
                0,
                &make_map(vec![Tile {
                    x: 0,
                    y: 0,
                    tile_type: crate::schemas::TileType::Floor
                }]),
                &vec![],
                TileType::is_blocking_sight
            ),
            Ok(())
        )
    }

    #[test]
    fn test_has_no_los_diagonal() {
        assert_eq!(
            has_los(
                -2,
                0,
                3,
                1,
                &make_map(vec![Tile {
                    x: 0,
                    y: 0,
                    tile_type: crate::schemas::TileType::Wall
                }]),
                &vec![],
                TileType::is_blocking_sight
            ),
            Err(Tile {
                x: 0,
                y: 0,
                tile_type: crate::schemas::TileType::Wall
            })
        )
    }

    #[sqlx::test]
    async fn test_get_gamestate(pool: sqlx::Pool<Sqlite>) {
        let repo = Repository::from_pool(pool).await;
        assert_eq!(
            super::get_gamestate(repo, 1)
                .await
                .expect("Error while getting gamestate"),
            Gamestate {
                id: 1,
                playing: 1,
                logs: vec![],
                allied_vision: HashSet::new(),
                abilities: vec![
                    AbilityTargets {
                        name: AbilityName::Move,
                        targets: HashSet::from_iter(
                            vec![Coords { x: -1, y: 1 }, Coords { x: -1, y: -1 }].into_iter()
                        ),
                        costs: vec![]
                    },
                    AbilityTargets {
                        name: AbilityName::Attack,
                        targets: HashSet::from_iter(vec![].into_iter()),
                        costs: vec![]
                    },
                    AbilityTargets {
                        name: AbilityName::Wait,
                        targets: HashSet::from_iter(vec![Coords { x: -2, y: 0 }].into_iter()),
                        costs: vec![]
                    },
                    AbilityTargets {
                        name: AbilityName::ShieldBash,
                        targets: HashSet::from_iter(vec![].into_iter()),
                        costs: vec![("ShieldBash".to_string(), 60.0)]
                    },
                ],
                entities: vec![EntityWithResources {
                    entity: Entity {
                        id: 1,
                        x: -2,
                        y: 0,
                        user_id: "diane".to_string(),
                        last_move_time: 0,
                        next_move_time: 0,
                        scenario_player_id: 1,
                        game_class: "Warrior".to_string(),
                    },
                    resources: vec![Resource {
                        resource_current: 100.0,
                        entity_id: 1,
                        game_id: 1,
                        resource_name: "hp".to_string(),
                        resource_max: 100.0,
                        resource_per_turn: 0.1
                    }]
                },],
                visible_tiles: HashSet::from_iter(
                    vec![
                        Tile {
                            x: -1,
                            y: 1,
                            tile_type: TileType::Floor,
                        },
                        Tile {
                            x: -1,
                            y: -1,
                            tile_type: TileType::Floor
                        },
                        Tile {
                            x: -2,
                            y: 0,
                            tile_type: TileType::Floor
                        },
                        Tile {
                            x: 0,
                            y: 0,
                            tile_type: TileType::Wall
                        }
                    ]
                    .into_iter()
                )
            }
        )
    }

    #[sqlx::test]
    async fn test_move(pool: sqlx::Pool<Sqlite>) {
        let repo = Repository::from_pool(pool).await;
        super::use_ability(
            repo.clone(),
            Events::new(),
            1,
            "diane".to_string(),
            AbilityName::Move,
            Coords { x: -1, y: 1 },
        )
        .await
        .expect("Move should succeed here");
        assert_eq!(
            super::get_gamestate(repo, 1)
                .await
                .expect("Error while getting gamestate"),
            Gamestate {
                id: 1,
                playing: 2,
                logs: vec![],
                allied_vision: HashSet::new(),
                abilities: vec![
                    AbilityTargets {
                        name: AbilityName::Move,
                        targets: HashSet::from_iter(
                            vec![Coords { x: 1, y: -1 }, Coords { x: 1, y: 1 }].into_iter(),
                        ),
                        costs: vec![]
                    },
                    AbilityTargets {
                        name: AbilityName::Attack,
                        targets: HashSet::from_iter(vec![].into_iter()),
                        costs: vec![]
                    },
                    AbilityTargets {
                        name: AbilityName::Wait,
                        targets: HashSet::from_iter(vec![Coords { x: 2, y: 0 }].into_iter()),
                        costs: vec![]
                    },
                    AbilityTargets {
                        name: AbilityName::ShieldBash,
                        targets: HashSet::from_iter(vec![].into_iter()),
                        costs: vec![("ShieldBash".to_string(), 60.0)]
                    },
                ],
                entities: vec![EntityWithResources {
                    entity: Entity {
                        id: 2,
                        x: 2,
                        y: 0,
                        user_id: "arthur".to_string(),
                        last_move_time: 0,
                        next_move_time: 0,
                        game_class: "Warrior".to_string(),
                        scenario_player_id: 2,
                    },
                    resources: vec![Resource {
                        resource_current: 100.0,
                        entity_id: 2,
                        game_id: 1,
                        resource_name: "hp".to_string(),
                        resource_max: 100.0,
                        resource_per_turn: 0.1
                    }]
                }],
                visible_tiles: HashSet::from_iter(
                    vec![
                        Tile {
                            x: 1,
                            y: -1,
                            tile_type: TileType::Floor
                        },
                        Tile {
                            x: 1,
                            y: 1,
                            tile_type: TileType::Floor
                        },
                        Tile {
                            x: 2,
                            y: 0,
                            tile_type: TileType::Floor
                        },
                        Tile {
                            x: 0,
                            y: 0,
                            tile_type: TileType::Wall
                        }
                    ]
                    .into_iter()
                ),
            }
        )
    }
}
