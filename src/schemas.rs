use std::{collections::HashSet, str::FromStr};

use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::{abilities::AbilityName, charclasses::CharClass, stores::database::MapTile};

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterForm {
    pub username: String,
    pub password: String,
    pub confirm: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct AbilityTargets {
    pub name: AbilityName,
    pub targets: HashSet<Coords>,
    pub costs: Vec<(String, f64)>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Gamestate {
    pub id: i64,
    pub playing: i64,
    pub entities: Vec<EntityWithResources>,
    pub abilities: Vec<AbilityTargets>,
    pub visible_tiles: HashSet<Tile>,
    pub allied_vision: HashSet<Tile>,
    pub logs: Vec<ActionLogResponse>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, PartialEq, Eq, Hash, Clone)]
pub struct Coords {
    pub x: i64,
    pub y: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScenarioPlayer {
    pub id: i64,
    pub player_points: i64,
    pub drop_tiles: Vec<Coords>,
    pub allowed_clases: Vec<AvailableClass>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, PartialEq, Clone)]
pub struct ActionLog {
    pub game_id: i64,
    pub turn_time: i64,
    pub caster: i64,
    pub target_entity: Option<i64>,
    pub action_name: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow, PartialEq, Clone)]
pub struct ActionLogResponse {
    pub turn_time: i64,
    pub caster: Option<i64>,
    pub target_entity: Option<i64>,
    pub action_name: AbilityName,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct SimpleLog {
    pub is_friendly: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TargetCoordRequest {
    pub x: i64,
    pub y: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow, PartialEq, Clone)]
pub struct Resource {
    pub entity_id: i64,
    pub game_id: i64,
    pub resource_name: String,
    pub resource_max: f64,
    pub resource_current: f64,
    pub resource_per_turn: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AttackEntityRequest {
    pub entity_id: i64,
}
#[derive(Debug, Serialize, Deserialize, FromRow, PartialEq, Clone)]
pub struct Entity {
    pub id: i64,
    pub user_id: String,
    pub scenario_player_id: i64,
    pub last_move_time: i64,
    pub next_move_time: i64,
    pub game_class: String,
    pub x: i64,
    pub y: i64,
}

impl Entity {
    pub fn get_class(&self) -> CharClass {
        return CharClass::from_str(self.game_class.as_str()).unwrap();
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct EntityWithResources {
    pub entity: Entity,
    pub resources: Vec<Resource>,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub enum TileType {
    Floor,
    Wall,
    TallGrass,
    DeepWater,
}

impl FromStr for TileType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Floor" => Ok(TileType::Floor),
            "Wall" => Ok(TileType::Wall),
            "TallGrass" => Ok(TileType::TallGrass),
            "DeepWater" => Ok(TileType::DeepWater),
            _ => Err("Unknown tile type".to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Hash, PartialEq, Eq, Clone)]
pub struct Tile {
    pub x: i64,
    pub y: i64,
    pub tile_type: TileType,
}

impl From<MapTile> for Tile {
    fn from(value: MapTile) -> Self {
        Self {
            x: value.x,
            y: value.y,
            tile_type: TileType::from_str(value.tile_type.as_str()).unwrap(),
        }
    }
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

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct AvailableClass {
    pub game_class: String,
    pub player_points: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow, PartialEq)]
pub struct ShortEntity {
    pub game_class: String,
    pub x: i64,
    pub y: i64,
}

impl ShortEntity {
    pub fn get_class(&self) -> CharClass {
        return CharClass::from_str(self.game_class.as_str()).unwrap();
    }
}

impl From<Entity> for ShortEntity {
    fn from(value: Entity) -> Self {
        Self {
            x: value.x,
            game_class: value.game_class,
            y: value.y,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct DeployEntitiesRequest {
    pub scenario_player_id: i64,
    pub entities: Vec<ShortEntity>,
}
