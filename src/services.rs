use std::{
    collections::{HashMap, HashSet},
    hash::RandomState,
};

use uuid::Uuid;

use crate::{
    abilities::{Ability, AbilityName, TargetType},
    charclasses::{self, CharClass},
    schemas::{
        AbilityTargets, ActionLog, AvailableClass, Coords, DeployEntitiesRequest, Entity,
        EntityResponse, Game, GameRef, GameStatus, Gamestate, ScenarioPlayer, TileType,
    },
    stores::{database::Repository, events::Events},
};

#[derive(Debug)]
pub enum ServiceError {
    StorageError(String),
    BadRequest(String),
    QueueError(String),
    NotFound,
    Unauthorized,
}

impl From<password_hash::Error> for ServiceError {
    fn from(_value: password_hash::Error) -> Self {
        return Self::Unauthorized;
    }
}

impl From<std::io::Error> for ServiceError {
    fn from(value: std::io::Error) -> Self {
        return Self::StorageError(value.to_string());
    }
}
impl From<rmp_serde::encode::Error> for ServiceError {
    fn from(value: rmp_serde::encode::Error) -> Self {
        return Self::StorageError(value.to_string());
    }
}
impl From<rmp_serde::decode::Error> for ServiceError {
    fn from(value: rmp_serde::decode::Error) -> Self {
        return Self::StorageError(value.to_string());
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
            Self::StorageError(err) => err.to_owned(),
            Self::BadRequest(err) | Self::QueueError(err) => err.to_owned(),
            Self::NotFound => "Not found".to_string(),
            Self::Unauthorized => "Unauthorized".to_string(),
        }
    }
}

pub fn get_gamestate(game: &Game) -> Result<Gamestate, ServiceError> {
    let to_play = game.get_trait_entity()?;
    let allied_entities = game.allied_entities(to_play.scenario_player_index);
    let blocking_entities = game.blocking_entities(to_play.scenario_player_index);
    let (los_tiles, allied_vision) = get_los_map(
        &to_play.coords,
        &allied_entities,
        &game.map,
        &blocking_entities,
    );
    tracing::info!("game {:?}", game);
    let visible_entities: HashMap<Coords, Vec<EntityResponse>, RandomState> = HashMap::from_iter(
        game.entities
            .clone()
            .into_iter()
            .map(|(coords, vec_e)| {
                (
                    coords,
                    vec_e
                        .into_iter()
                        .filter(|e| {
                            allied_vision.contains(&e.coords)
                                || los_tiles.contains(&e.coords)
                                || e.scenario_player_index == to_play.scenario_player_index
                        })
                        .map(|e| {
                            EntityResponse::from_entity(
                                e.clone(),
                                to_play,
                                &game.entities,
                                &los_tiles,
                            )
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .filter(|(_coords, vec_e)| vec_e.len() != 0),
    );
    Ok(Gamestate {
        id: game.id,
        entities: visible_entities,
        playing: to_play.id,
        abilities: to_play
            .game_class
            .get_ability_list()
            .into_iter()
            .map(|name| AbilityTargets {
                name: name.clone(),
                costs: Ability {
                    name: name.clone(),
                    caster: to_play.clone(),
                }
                .get_costs(),
                targets: los_tiles
                    .clone()
                    .into_iter()
                    .filter(|tile| {
                        let default = vec![];
                        let target_entity = game.entities.get(tile).unwrap_or(&default).get(0);
                        {
                            is_valid_target(
                                &Ability {
                                    name: name.clone(),
                                    caster: to_play.clone(),
                                },
                                &tile,
                                target_entity,
                                &blocking_entities,
                                &game.map,
                            )
                            .is_ok()
                        }
                    })
                    .collect(),
            })
            .collect(),
        visible_tiles: los_tiles,
        allied_vision,
    })
}

pub fn get_distance(start: &Coords, end: &Coords) -> f64 {
    f64::sqrt(
        (((start.x - end.x) * (start.x - end.x)) as f64 / 4.0)
            + (((start.y - end.y) * (start.y - end.y)) as f64 * 3.0 / 4.0),
    )
}

pub async fn transfer_entity(
    repo: Repository,
    game_id: uuid::Uuid,
    user_from: String,
    user_to: String,
) -> Result<(), ServiceError> {
    let mut game = repo.load_game(&game_id).await?;
    let entity = game.get_trait_entity_mut()?;
    if entity.user_id != user_from {
        return Err(ServiceError::Unauthorized);
    }
    entity.user_id = user_to;
    repo.save_game(&game).await?;
    Ok(())
}

pub async fn deploy_entities(
    repo: Repository,
    events: Events,
    user_id: String,
    game_id: uuid::Uuid,
    req: DeployEntitiesRequest,
) -> Result<(), ServiceError> {
    // Check if game is Open
    // Check if scenario_player_id already deployed
    // Check points
    // Check location
    let mut game = repo.load_game(&game_id).await?;
    let count_indices = HashSet::<i64, RandomState>::from_iter(
        game.entities
            .values()
            .flatten()
            .map(|e| e.scenario_player_index),
    )
    .len();

    for (coords, class) in req.entities {
        game.entities.insert(
            coords.clone(),
            vec![Entity {
                user_id: user_id.clone(),
                id: uuid::Uuid::new_v4(),
                coords: coords,
                resources: HashMap::from_iter(class.get_resource_list()),
                scenario_player_index: req.scenario_player_id,
                last_move_time: 0,
                next_move_time: 0,
                game_class: class,
                log: vec![],
            }],
        );
    }
    if count_indices >= 1 {
        let mut game_list = repo.load_game_list().await?;
        game_list.get_mut(&game_id).unwrap().status = GameStatus::Running;
        repo.save_game_list(game_list).await?;
        let _res = tick(events, &game).await;
    };
    repo.save_game(&game).await?;
    Ok(())
}

pub async fn tick(events: Events, game: &Game) -> Result<(), ServiceError> {
    let to_play = game.get_trait_entity()?;
    tracing::info!("sending tick to {}", to_play.user_id);
    events
        .send_event(get_gamestate(game)?, game.id, to_play.user_id.clone())
        .await
}

pub fn get_possible_moves(start: &Coords, end: &Coords) -> Vec<Coords> {
    let diff_x = end.x - start.x;
    let diff_y = end.y - start.y;
    if (diff_x, diff_y) == (0, 0) {
        return Vec::new();
    }
    if diff_y == 0 {
        if diff_x < 0 {
            return vec![Coords { x: -2, y: 0 }];
        }
        return vec![Coords { x: 2, y: 0 }];
    }
    if diff_y > 0 {
        if diff_x > 0 {
            if diff_y > diff_x {
                return vec![Coords { x: -1, y: 1 }, Coords { x: 1, y: 1 }];
            } else if diff_y < diff_x {
                return vec![Coords { x: 2, y: 0 }, Coords { x: 1, y: 1 }];
            } else {
                return vec![Coords { x: 1, y: 1 }];
            }
        } else {
            if diff_y > -diff_x {
                return vec![Coords { x: -1, y: 1 }, Coords { x: 1, y: 1 }];
            } else if diff_y < -diff_x {
                return vec![Coords { x: -2, y: 0 }, Coords { x: -1, y: 1 }];
            } else {
                return vec![Coords { x: -1, y: 1 }];
            }
        }
    } else {
        if diff_x > 0 {
            if -diff_y > diff_x {
                return vec![Coords { x: -1, y: -1 }, Coords { x: 1, y: -1 }];
            } else if -diff_y < diff_x {
                return vec![Coords { x: 2, y: 0 }, Coords { x: 1, y: -1 }];
            } else {
                return vec![Coords { x: 1, y: -1 }];
            }
        } else {
            if -diff_y > -diff_x {
                return vec![Coords { x: -1, y: -1 }, Coords { x: 1, y: -1 }];
            } else if -diff_y < -diff_x {
                return vec![Coords { x: -2, y: 0 }, Coords { x: -1, y: -1 }];
            } else {
                return vec![Coords { x: -1, y: -1 }];
            }
        }
    }
}

pub fn get_los_line(start: &Coords, end: &Coords) -> HashSet<Coords> {
    let mut init_pos = HashSet::new();
    init_pos.insert(start.clone());
    if start == end {
        return init_pos;
    }
    let max_dist = get_distance(start, end) + 1.001;
    let moves = get_possible_moves(start, end);
    let mut reached_end = false;

    loop {
        let mut new_pos = HashSet::new();
        for tile in init_pos.clone() {
            for mv in moves.clone() {
                let new_tile = Coords {
                    x: tile.x + mv.x,
                    y: tile.y + mv.y,
                };
                if get_distance(&new_tile, start) + get_distance(&new_tile, end) > max_dist {
                    continue;
                }
                if &new_tile == end {
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
    start: &Coords,
    end: &Coords,
    map: &HashMap<Coords, TileType, RandomState>,
    blocking_entities: &Vec<&Entity>,
    is_blocking: fn(&TileType) -> bool,
) -> Result<(), Coords> {
    let dist_start_end = get_distance(start, end);
    if dist_start_end < 0.01 {
        return Ok(());
    }
    let mut coords: Vec<Coords> = get_los_line(start, end).into_iter().collect();
    let blocked_coords: HashSet<Coords, RandomState> =
        HashSet::from_iter(blocking_entities.into_iter().map(|e| e.coords.clone()));
    coords.sort_by(|a, b| {
        get_distance(&start, a)
            .partial_cmp(&get_distance(&start, b))
            .unwrap()
    });
    for coord in coords {
        let distance_to_line = ((((end.x - start.x) as f64 / 2.0)
            * (coord.y - start.y) as f64
            * (0.75_f64).sqrt())
            - ((coord.x - start.x) as f64 / 2.0) * ((end.y - start.y) as f64 * (0.75_f64).sqrt()))
        .abs();
        if distance_to_line > dist_start_end / 2.0 {
            continue;
        }
        let is_blocked = blocked_coords.get(&coord);
        if is_blocked.is_some() {
            return Err(coord);
        }
        match map.get(&coord) {
            None => continue,
            Some(tile_type) => {
                if is_blocking(&tile_type) {
                    return Err(coord);
                }
            }
        }
    }
    Ok(())
}

pub fn get_los_map(
    from_point: &Coords,
    allied_entities: &Vec<&Entity>,
    map: &HashMap<Coords, TileType>,
    blocking_entities: &Vec<&Entity>,
) -> (HashSet<Coords>, HashSet<Coords>) {
    let mut allied_vision = HashSet::new();
    let mut los_map = HashSet::new();
    for coords in map.keys() {
        let los_result = has_los(
            from_point,
            &coords,
            map,
            blocking_entities,
            TileType::is_blocking_sight,
        );
        los_map.insert(match los_result {
            Ok(_) => coords.clone(),
            Err(wall) => {
                for ally in allied_entities.into_iter() {
                    let ally_tile = match has_los(
                        &ally.coords,
                        &coords,
                        &map,
                        blocking_entities,
                        TileType::is_blocking_sight,
                    ) {
                        Ok(_) => coords.clone(),
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

pub async fn get_active_game(repo: Repository, user_id: String) -> Result<Vec<Uuid>, ServiceError> {
    let game_list = repo.load_game_list().await?;
    Ok(game_list
        .keys()
        .filter(|k| {
            let game_ref = game_list.get(k).unwrap();
            game_ref.status == GameStatus::Open || game_ref.seated_players.contains(&user_id)
        })
        .map(|k| k.clone())
        .collect())
}

pub fn get_scenarios() -> Vec<i64> {
    vec![0]
}

pub fn get_scenario_players() -> Vec<ScenarioPlayer> {
    let allowed_classes = vec![
        AvailableClass {
            player_points: 25,
            game_class: CharClass::Archer,
        },
        AvailableClass {
            player_points: 25,
            game_class: CharClass::Warrior,
        },
    ];
    vec![
        ScenarioPlayer {
            player_points: 100,
            drop_tiles: vec![Coords { x: -2, y: 0 }],
            allowed_clases: allowed_classes.clone(),
        },
        ScenarioPlayer {
            player_points: 100,
            drop_tiles: vec![Coords { x: 2, y: 0 }],
            allowed_clases: allowed_classes,
        },
    ]
}

pub async fn get_available_scenario_players(
    repo: Repository,
    game_id: uuid::Uuid,
) -> Result<Vec<ScenarioPlayer>, ServiceError> {
    let allowed_classes = vec![
        AvailableClass {
            player_points: 25,
            game_class: CharClass::Archer,
        },
        AvailableClass {
            player_points: 25,
            game_class: CharClass::Warrior,
        },
    ];
    Ok(vec![
        ScenarioPlayer {
            player_points: 100,
            drop_tiles: vec![Coords { x: -2, y: 0 }],
            allowed_clases: allowed_classes.clone(),
        },
        ScenarioPlayer {
            player_points: 100,
            drop_tiles: vec![Coords { x: 2, y: 0 }],
            allowed_clases: allowed_classes,
        },
    ])
}

pub async fn new_game(repo: Repository) -> Result<uuid::Uuid, ServiceError> {
    let game = Game::new();
    repo.save_game(&game).await?;
    let mut game_list = match repo.load_game_list().await {
        Err(_) => HashMap::new(),
        Ok(list) => list,
    };
    game_list.insert(
        game.id,
        GameRef {
            game_id: game.id,
            seated_players: vec![],
            status: GameStatus::Open,
            scenario: 0,
        },
    );
    repo.save_game_list(game_list).await?;
    Ok(game.id)
}

pub fn is_valid_target(
    ability: &Ability,
    target: &Coords,
    target_entity: Option<&Entity>,
    blocking_entities: &Vec<&Entity>,
    map: &HashMap<Coords, TileType>,
) -> Result<(), ServiceError> {
    let distance = get_distance(&ability.caster.coords, target);
    if distance > ability.max_range(&ability.caster.game_class) {
        return Err(ServiceError::BadRequest("Out of range".to_string()));
    }
    match ability.target_type() {
        TargetType::Walkable => {
            if target_entity.is_some() {
                return Err(ServiceError::BadRequest(
                    "Target must be walkable".to_string(),
                ));
            }
            match map.get(target) {
                Some(TileType::Wall) | Some(TileType::DeepWater) => {
                    return Err(ServiceError::BadRequest(
                        "Can't walk on that tile".to_string(),
                    ))
                }
                _ => {}
            }
        }
        TargetType::Ennemy => {
            if target_entity.is_none()
                || target_entity.clone().is_some_and(|e| {
                    e.scenario_player_index == ability.caster.scenario_player_index
                })
            {
                return Err(ServiceError::BadRequest(
                    "Target must be an ennemy".to_string(),
                ));
            }
        }
        TargetType::Ally => {
            if target_entity.is_none()
                || target_entity.clone().is_some_and(|e| {
                    e.scenario_player_index != ability.caster.scenario_player_index
                })
            {
                return Err(ServiceError::BadRequest(
                    "Target must be an ally".to_string(),
                ));
            }
        }
        TargetType::Selfcast => {
            if target_entity != Some(&ability.caster) {
                return Err(ServiceError::BadRequest(
                    "This ability is self-cast".to_string(),
                ));
            }
        }
    }
    if ability.needs_los() {
        if has_los(
            &ability.caster.coords,
            target,
            map,
            blocking_entities,
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
    game_id: uuid::Uuid,
    user_id: String,
    ability_name: AbilityName,
    target: Coords,
) -> Result<(), ServiceError> {
    let game = repo.load_game(&game_id).await?;
    let entity = game.get_trait_entity()?;
    let current_time = entity.next_move_time;
    if entity.user_id != user_id {
        return Err(ServiceError::Unauthorized);
    }
    let target_entity = game.entities.get(&target).map(|v| v.get(0)).unwrap_or(None);
    let ability = Ability {
        name: ability_name.clone(),
        caster: entity.clone(),
    };

    let blocking_entities = game.blocking_entities(entity.scenario_player_index);
    is_valid_target(
        &ability,
        &target,
        target_entity,
        &blocking_entities,
        &game.map,
    )?;
    let costs = ability.get_costs();
    for (resource_name, cost) in costs.clone() {
        let resource = entity.resources.get(&resource_name);
        if resource.is_none() || resource.unwrap().current < cost {
            return Err(ServiceError::BadRequest(format!(
                "Ability is not ready, lack resource {}",
                resource_name
            )));
        }
    }
    let mut mut_game = game.clone();
    mut_game.apply_ability(&ability, &target);

    let new_entity = mut_game.get_trait_entity()?;
    let elapsed_time = new_entity.next_move_time - current_time;
    tracing::info!("elapsed {:?}", elapsed_time);

    mut_game.increment_resources(elapsed_time);
    let gamestate = get_gamestate(&mut_game)?;
    tracing::info!("{:?}", gamestate);
    let _res = events.send_event(gamestate, game_id, user_id).await;
    repo.save_game(&mut_game).await?;
    Ok(())
}
