use std::{
    cmp::max,
    collections::{HashMap, HashSet},
};

use serde::{Deserialize, Serialize};
use uuid;

use crate::{
    abilities::{Ability, AbilityName},
    charclasses::CharClass,
    services::ServiceError,
};
use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use password_hash::rand_core::OsRng;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct GameRef {
    pub game_id: uuid::Uuid,
    pub seated_players: Vec<String>,
    pub status: GameStatus,
    pub scenario: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Game {
    pub id: uuid::Uuid,
    pub entities: HashMap<Coords, Vec<Entity>>,
    pub map: HashMap<Coords, TileType>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum GameStatus {
    Running,
    Open,
}

impl Game {
    pub fn get_trait_entity(&self) -> Result<&Entity, ServiceError> {
        match self.entities.values().flatten().reduce(|acc, e| {
            if e.next_move_time < acc.next_move_time {
                e
            } else {
                acc
            }
        }) {
            Some(entity) => Ok(entity),
            None => Err(ServiceError::BadRequest("No active entity".to_string())),
        }
    }
    pub fn get_trait_entity_mut(&mut self) -> Result<&mut Entity, ServiceError> {
        match self.entities.values_mut().flatten().reduce(|acc, e| {
            if e.next_move_time < acc.next_move_time {
                e
            } else {
                acc
            }
        }) {
            Some(entity) => Ok(entity),
            None => Err(ServiceError::BadRequest("No active entity".to_string())),
        }
    }
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            entities: HashMap::new(),
            map: HashMap::new(), // todo load map
        }
    }
    pub fn blocking_entities(&self, index: i64) -> Vec<&Entity> {
        self.entities
            .values()
            .flatten()
            .filter(|e| e.scenario_player_index != index)
            .collect()
    }
    pub fn allied_entities(&self, index: i64) -> Vec<&Entity> {
        self.entities
            .values()
            .flatten()
            .filter(|e| e.scenario_player_index == index)
            .collect()
    }

    pub fn increment_resources(&mut self, elapsed_time: i64) {
        self.entities.values_mut().flatten().for_each(|e| {
            e.resources
                .values_mut()
                .for_each(|r| r.current = max(r.current + r.per_turn * elapsed_time, r.max));
        });
    }
    pub fn apply_ability(
        &mut self,
        ability: &Ability,
        target: &Coords,
    ) -> Result<(), ServiceError> {
        match ability.name {
            AbilityName::ShieldBash => {
                let target_entity: &mut Entity =
                    self.entities.get_mut(target).unwrap().get_mut(0).unwrap();
                target_entity.resources.get_mut("hp").unwrap().current -=
                    (ability.caster.game_class.get_attack_damage() as f64 * 0.8) as i64;
                target_entity.next_move_time += 12;
            }
            AbilityName::Move => {
                self.entities
                    .get_mut(&ability.caster.coords)
                    .unwrap()
                    .get_mut(0)
                    .unwrap()
                    .coords = target.clone();
            }
            AbilityName::Attack => {
                let target_entity: &mut Entity =
                    self.entities.get_mut(target).unwrap().get_mut(0).unwrap();
                target_entity.resources.get_mut("hp").unwrap().current -=
                    ability.caster.game_class.get_attack_damage();
            }
            AbilityName::Wait => {}
        }
        let game_caster = self
            .entities
            .get_mut(&ability.caster.coords)
            .unwrap()
            .get_mut(0)
            .unwrap();
        for (resource_name, cost) in ability.get_costs() {
            game_caster
                .resources
                .get_mut(&resource_name)
                .unwrap()
                .current -= cost;
        }
        game_caster.last_move_time = ability.caster.next_move_time;
        game_caster.next_move_time += ability.get_delay(target.clone());
        game_caster.log.push(ActionLog {
            turn_time: ability.caster.last_move_time,
            target: target.clone(),
            action_name: ability.name.clone(),
        });
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct UserData {
    pub roles: Vec<String>,
    pub id: String,
    pub passhash: String,
}

impl From<LoginForm> for UserData {
    fn from(value: LoginForm) -> Self {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(value.password.as_bytes(), &salt)
            .unwrap()
            .to_string();
        Self {
            roles: vec![],
            id: value.username,
            passhash: password_hash,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

impl LoginForm {
    pub fn verify(&self, data: &UserData) -> Result<(), ServiceError> {
        let parsed_hash = PasswordHash::new(&data.passhash)?;
        Ok(Argon2::default().verify_password(self.password.as_bytes(), &parsed_hash)?)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct AbilityTargets {
    pub name: AbilityName,
    pub targets: HashSet<Coords>,
    pub costs: Vec<(String, i64)>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Gamestate {
    pub id: uuid::Uuid,
    pub entities: HashMap<Coords, Vec<EntityResponse>>,
    pub abilities: Vec<AbilityTargets>,
    pub visible_tiles: HashSet<Coords>,
    pub allied_vision: HashSet<Coords>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct Coords {
    pub x: i64,
    pub y: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct ScenarioPlayer {
    pub player_points: i64,
    pub drop_tiles: Vec<Coords>,
    pub allowed_clases: Vec<AvailableClass>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct ActionLog {
    pub turn_time: i64,
    pub target: Coords,
    pub action_name: AbilityName,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct ActionLogResponse {
    pub turn_time: i64,
    pub caster: Entity,
    pub target_entity: Option<EntityResponse>,
    pub action_name: AbilityName,
}

impl ActionLogResponse {
    fn from_log(
        action_log: &Vec<ActionLog>,
        caster: Entity,
        to_play: &Entity,
        entities: &HashMap<Coords, Vec<Entity>>,
        visible_tiles: &HashSet<Coords>,
    ) -> Vec<Self> {
        action_log
            .into_iter()
            .filter(|l| l.turn_time >= to_play.last_move_time && visible_tiles.contains(&l.target))
            .map(|log| Self {
                turn_time: log.turn_time,
                caster: caster.clone(),
                action_name: log.action_name.clone(),
                target_entity: entities
                    .get(&log.target)
                    .unwrap_or(&vec![])
                    .get(0)
                    .cloned()
                    .map(|entity| {
                        EntityResponse::from_entity(entity, to_play, entities, visible_tiles)
                    }),
            })
            .collect()
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Resource {
    pub max: i64,
    pub current: i64,
    pub per_turn: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AttackEntityRequest {
    pub entity_id: i64,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Entity {
    pub user_id: String,
    pub coords: Coords,
    pub resources: HashMap<String, Resource>,
    pub scenario_player_index: i64,
    pub last_move_time: i64,
    pub next_move_time: i64,
    pub game_class: CharClass,
    pub log: Vec<ActionLog>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct EntityResponse {
    pub coords: Coords,
    pub playing: bool,
    pub resources: HashMap<String, Resource>,
    pub scenario_player_index: i64,
    pub next_move_time: i64,
    pub game_class: CharClass,
    pub log: Vec<ActionLogResponse>,
}

impl EntityResponse {
    pub fn from_entity(
        value: Entity,
        to_play: &Entity,
        entities: &HashMap<Coords, Vec<Entity>>,
        visible_tiles: &HashSet<Coords>,
    ) -> Self {
        Self {
            coords: value.coords.clone(),
            playing: &value == to_play,
            resources: value.resources.clone(),
            scenario_player_index: value.scenario_player_index,
            next_move_time: value.next_move_time,
            game_class: value.game_class.clone(),
            log: ActionLogResponse::from_log(
                &value.log,
                value.clone(),
                &to_play.clone(),
                &entities,
                visible_tiles,
            ),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub enum TileType {
    Floor,
    Wall,
    TallGrass,
    DeepWater,
}

impl TileType {
    pub fn is_blocking_sight(&self) -> bool {
        match self {
            Self::TallGrass | Self::Wall => true,
            _ => false,
        }
    }
    pub fn is_blocking_walk(&self) -> bool {
        match self {
            Self::DeepWater | Self::Wall => true,
            _ => false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct AvailableClass {
    pub game_class: CharClass,
    pub player_points: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeployEntitiesRequest {
    pub scenario_player_id: i64,
    pub entities: Vec<(Coords, CharClass)>,
}
