use crate::schemas::{Game, GameRef, UserData};
use crate::services::ServiceError;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone)]
pub struct Repository {}

impl Repository {
    pub async fn new() -> Self {
        Self {}
    }
    pub fn game_file_name(game_id: &uuid::Uuid) -> String {
        format!("game_{}.mp", game_id)
    }

    pub fn user_file_name(user_id: &str) -> String {
        format!("user_{}.mp", user_id)
    }

    pub async fn save_game_list(
        &self,
        game_list: HashMap<uuid::Uuid, GameRef>,
    ) -> Result<(), ServiceError> {
        let file = fs::File::create("game_list.mp")?;
        game_list.serialize(&mut rmp_serde::Serializer::new(file))?;
        Ok(())
    }

    pub async fn load_game_list(&self) -> Result<HashMap<uuid::Uuid, GameRef>, ServiceError> {
        let file = fs::File::open("game_list.mp")?;
        Ok(rmp_serde::from_read(file)?)
    }

    pub async fn save_game(&self, game: &Game) -> Result<(), ServiceError> {
        let file = fs::File::create(Self::game_file_name(&game.id).as_str())?;
        game.serialize(&mut rmp_serde::Serializer::new(file))?;
        Ok(())
    }

    pub async fn load_game(&self, game_id: &uuid::Uuid) -> Result<Game, ServiceError> {
        let file = fs::File::open(Self::game_file_name(game_id).as_str())?;
        Ok(rmp_serde::from_read(file)?)
    }

    pub async fn unlink_game(&self, game_id: &uuid::Uuid) -> Result<(), ServiceError> {
        Ok(fs::remove_file(Self::game_file_name(game_id))?)
    }

    pub async fn save_user(&self, user: &UserData) -> Result<(), ServiceError> {
        let file = fs::File::create(Self::user_file_name(&user.id).as_str())?;
        user.serialize(&mut rmp_serde::Serializer::new(file))?;
        Ok(())
    }

    pub async fn load_user(&self, user_id: &str) -> Result<UserData, ServiceError> {
        let file = fs::File::open(Self::user_file_name(user_id).as_str())?;
        Ok(rmp_serde::from_read(file)?)
    }

    pub async fn unlink_user(&self, user_id: &str) -> Result<(), ServiceError> {
        Ok(fs::remove_file(Self::user_file_name(user_id))?)
    }
}
