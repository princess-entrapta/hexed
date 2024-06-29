use std::collections::HashMap;
use std::hash::RandomState;
use std::str::FromStr;

use argon2::{password_hash::SaltString, Argon2, PasswordHasher, PasswordVerifier};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use sqlx::{Pool, Row, Sqlite};

use crate::abilities::AbilityName;
use crate::charclasses::ResourceStat;
use crate::schemas::{
    ActionLog, AvailableClass, Coords, Entity, EntityWithResources, Resource, ScenarioPlayer,
    ShortEntity, Tile,
};

#[derive(Debug, FromRow, Clone, Serialize, Deserialize)]
pub struct Game {
    id: i64,
    game_status: String,
    user_owner: String,
    scenario_description: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct MapTile {
    pub x: i64,
    pub y: i64,
    pub tile_type: String,
}

#[derive(Debug, FromRow, Clone)]

pub struct UserData {
    pub roles: String,
    pub passhash: String,
}

#[derive(Debug, Clone)]
pub struct Repository {
    pool: Pool<Sqlite>,
}

impl Repository {
    pub async fn new() -> Self {
        Self {
            pool: SqlitePool::connect_with(
                SqliteConnectOptions::new()
                    .filename("example.sqlite")
                    .create_if_missing(true),
            )
            .await
            .unwrap(),
        }
    }

    pub async fn from_pool(pool: sqlx::Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn get_resource(
        &self,
        game_id: &i64,
        entity_id: &i64,
        resource_name: &str,
    ) -> Result<Resource, sqlx::error::Error> {
        Ok(sqlx::query_as::<_, Resource>(
            r#"
        SELECT * FROM entity_resource WHERE game_id = ? AND entity_id = ? AND resource_name = ?
        "#,
        )
        .bind(game_id)
        .bind(entity_id)
        .bind(resource_name)
        .fetch_one(&self.pool)
        .await?)
    }

    pub async fn get_entity_resources(
        &self,
        game_id: &i64,
        entity_id: &i64,
    ) -> Result<Vec<Resource>, sqlx::error::Error> {
        Ok(sqlx::query_as::<_, Resource>(
            r#"
        SELECT * FROM entity_resource WHERE game_id = ? AND entity_id = ?
        "#,
        )
        .bind(game_id)
        .bind(entity_id)
        .fetch_all(&self.pool)
        .await?)
    }

    pub async fn add_resource(
        &self,
        entity_id: &i64,
        game_id: &i64,
        stats: &ResourceStat,
    ) -> Result<(), sqlx::error::Error> {
        sqlx::query(
            r#"INSERT INTO entity_resource (game_id, entity_id, resource_name, resource_current, resource_max, resource_per_turn) VALUES (?, ?, ?, ?, ?, ?);"#,
        )
        .bind(game_id)
        .bind(entity_id)
        .bind(stats.resource_name.clone())
        .bind(stats.resource_current.clone())
        .bind(stats.resource_max)
        .bind(stats.resource_per_turn)
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn increment_resources(
        &self,
        game_id: &i64,
        elapsed_time: &i64,
    ) -> Result<(), sqlx::error::Error> {
        sqlx::query(
            r#"
        UPDATE entity_resource SET resource_current = MIN(resource_current + ? * resource_per_turn, resource_max) WHERE game_id = ?
        "#,
        )
        .bind(elapsed_time)
        .bind(game_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn remove_entity(
        &self,
        game_id: &i64,
        entity_id: &i64,
    ) -> Result<(), sqlx::error::Error> {
        sqlx::query(
            r#"
        DELETE FROM entity WHERE game_id = ? AND id = ?;
        "#,
        )
        .bind(game_id)
        .bind(entity_id)
        .execute(&self.pool)
        .await?;
        sqlx::query(
            r#"
        DELETE FROM entity_resource WHERE game_id = ? AND entity_id = ?;
        "#,
        )
        .bind(game_id)
        .bind(entity_id)
        .execute(&self.pool)
        .await?;

        //TODO

        Ok(())
    }

    pub async fn get_scenarios(&self) -> Result<Vec<i64>, sqlx::error::Error> {
        Ok(
            sqlx::query(r#"SELECT DISTINCT scenario_id FROM scenario_player"#)
                .fetch_all(&self.pool)
                .await?
                .into_iter()
                .map(|r| r.get(0))
                .collect(),
        )
    }

    pub async fn get_available_scenario_players(
        &self,
        game_id: &i64,
    ) -> Result<Vec<ScenarioPlayer>, sqlx::error::Error> {
        let mut scenario_players: HashMap<i64, ScenarioPlayer, RandomState> = HashMap::new();
        sqlx::query(
            r#"SELECT scenario_player.id, x, y, player_points 
                            FROM scenario_player
                                JOIN game ON game.scenario_id = scenario_player.scenario_id
                                LEFT JOIN seated_player ON (seated_player.scenario_player_id = scenario_player.id AND game.id = seated_player.game_id)
                                JOIN drop_zone_tile ON drop_zone_tile.scenario_player_id = scenario_player.id 
                            WHERE game.id = ?
                              AND seated_player.scenario_player_id IS NULL"#,
        )
        .bind(game_id)
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .for_each(|row| {
            let id: i64 = row.get(0);
            if scenario_players.contains_key(&id) {
                scenario_players
                    .get_mut(&id)
                    .unwrap()
                    .drop_tiles
                    .push(Coords {
                        x: row.get(1),
                        y: row.get(2),
                    });
            } else {
                scenario_players.insert(
                    id,
                    ScenarioPlayer {
                        allowed_clases: Vec::new(),
                        id: row.get(0),
                        player_points: row.get(3),
                        drop_tiles: vec![Coords {
                            x: row.get(1),
                            y: row.get(2),
                        }],
                    },
                );
            }
        });
        sqlx::query(
            r#"SELECT scenario_player.id, game_class, available_class.player_points FROM available_class JOIN scenario_player ON scenario_player_id = scenario_player.id JOIN game ON game.scenario_id = scenario_player.scenario_id WHERE game.id = ?"#,
        )
        .bind(game_id)
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .for_each(|row| {
            scenario_players
                .get_mut(&row.get::<i64, _>(0))
                .map(|scenario_player| {scenario_player.allowed_clases.push(AvailableClass{game_class: row.get::<String, _>(1), player_points: row.get::<i64, _>(2)})});
        });
        Ok(scenario_players.values().map(|v| v.clone()).collect())
    }

    pub async fn get_scenario_players(
        &self,
        scenario_id: &i64,
    ) -> Result<Vec<ScenarioPlayer>, sqlx::error::Error> {
        let mut scenario_players: HashMap<i64, ScenarioPlayer, RandomState> = HashMap::new();
        sqlx::query(
            r#"SELECT scenario_player.id, x, y, player_points 
                            FROM scenario_player
                                JOIN drop_zone_tile ON drop_zone_tile.scenario_player_id = scenario_player.id 
                            WHERE scenario_id = ?"#,
        )
        .bind(scenario_id)
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .for_each(|row| {
            let id: i64 = row.get(0);
            if scenario_players.contains_key(&id) {
                scenario_players
                    .get_mut(&id)
                    .unwrap()
                    .drop_tiles
                    .push(Coords {
                        x: row.get(1),
                        y: row.get(2),
                    });
            } else {
                scenario_players.insert(
                    id,
                    ScenarioPlayer {
                        allowed_clases: Vec::new(),
                        id: row.get(0),
                        player_points: row.get(3),
                        drop_tiles: vec![Coords {
                            x: row.get(1),
                            y: row.get(2),
                        }],
                    },
                );
            }
        });
        sqlx::query(
            r#"SELECT scenario_player.id, game_class, available_class.player_points FROM available_class JOIN scenario_player ON scenario_player_id = scenario_player.id WHERE scenario_id = ?"#,
        )
        .bind(scenario_id)
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .for_each(|row| {
            scenario_players
                .get_mut(&row.get::<i64, _>(0))
                .unwrap()
                .allowed_clases
                .push(AvailableClass{game_class: row.get::<String, _>(1), player_points: row.get::<i64, _>(2)})
        });
        Ok(scenario_players.values().map(|v| v.clone()).collect())
    }

    pub async fn get_seated_player(
        &self,
        game_id: &i64,
        scenario_player_id: &i64,
    ) -> Result<String, sqlx::error::Error> {
        Ok(sqlx::query(
            r#"
        SELECT user_id FROM seated_player WHERE game_id = ? AND scenario_player_id = ?;
        "#,
        )
        .bind(game_id)
        .bind(scenario_player_id)
        .fetch_one(&self.pool)
        .await?
        .get(0))
    }

    pub async fn count_active_players(&self, game_id: &i64) -> Result<i64, sqlx::error::Error> {
        Ok(sqlx::query(
            r#"
        SELECT COUNT(*) FROM seated_player WHERE game_id = ?;
        "#,
        )
        .bind(game_id)
        .fetch_one(&self.pool)
        .await?
        .get(0))
    }

    pub async fn delete_game(&self, game_id: &i64) -> Result<(), sqlx::error::Error> {
        sqlx::query(
            r#"
        DELETE FROM entity WHERE game_id = ?;
        "#,
        )
        .bind(game_id)
        .execute(&self.pool)
        .await?;
        sqlx::query(
            r#"
        DELETE FROM entity_resource WHERE game_id = ?;
        "#,
        )
        .bind(game_id)
        .execute(&self.pool)
        .await?;
        sqlx::query(
            r#"
        DELETE FROM action_log WHERE game_id = ?;
        "#,
        )
        .bind(game_id)
        .execute(&self.pool)
        .await?;
        sqlx::query(
            r#"
        DELETE FROM game WHERE game_id = ?;
        "#,
        )
        .bind(game_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn open_games(&self) -> Result<Vec<Game>, sqlx::error::Error> {
        Ok(sqlx::query_as::<_, Game>(
            r#"
            SELECT game.id, game_status, user_owner, scenario.scenario_description 
                FROM game
                    JOIN scenario ON scenario.id = game.scenario_id 
                WHERE game_status = 'Open'"#,
        )
        .fetch_all(&self.pool)
        .await?)
    }

    pub async fn running_games(&self, user_id: &String) -> Result<Vec<Game>, sqlx::error::Error> {
        Ok(sqlx::query_as::<_, Game>(
            r#"
            SELECT game.id, game_status, seated_player.user_id AS user_owner, scenario.scenario_description
                FROM game
                    JOIN scenario ON scenario.id = game.scenario_id
                    JOIN seated_player ON seated_player.game_id = game.id
                WHERE game_status = 'Running' 
                  AND seated_player.user_id = ?"#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?)
    }

    pub async fn new_game(
        &self,
        user_id: &String,
        scenario_id: &i64,
    ) -> Result<i64, sqlx::error::Error> {
        Ok(sqlx::query(
            r#"INSERT INTO game (user_owner, game_status, scenario_id) VALUES (?, 'Open', ?) RETURNING id;"#,
        )
        .bind(user_id)
        .bind(scenario_id)
        .fetch_one(&self.pool)
        .await?
        .get::<i64, _>(0))
    }

    pub async fn set_resource(&self, resource: &Resource) -> Result<(), sqlx::error::Error> {
        sqlx::query(
            r#"
        UPDATE entity_resource SET resource_max = ?, resource_current = ?, resource_per_turn = ? WHERE game_id = ? AND entity_id = ? AND resource_name = ?;
        "#,
        )
        .bind(resource.resource_max)
        .bind(resource.resource_current)
        .bind(resource.resource_per_turn)
        .bind(resource.game_id)
        .bind(resource.entity_id)
        .bind(resource.resource_name.as_str())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn log_action(&self, log: &ActionLog) -> Result<(), sqlx::error::Error> {
        sqlx::query(
            r#"
                INSERT INTO action_log (
                    game_id,
                    turn_time,
                    action_name,
                    caster,
                    target_entity
                ) VALUES (?, ?, ?, ?, ?);
                "#,
        )
        .bind(log.game_id)
        .bind(log.turn_time)
        .bind(log.action_name.clone())
        .bind(log.caster)
        .bind(log.target_entity)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_logs_since(
        &self,
        game_id: &i64,
        turn_time: &i64,
    ) -> Result<Vec<ActionLog>, sqlx::error::Error> {
        Ok(sqlx::query_as::<_, ActionLog>(
            r#"SELECT * FROM action_log WHERE game_id = ? AND turn_time >= ? ORDER BY turn_time"#,
        )
        .bind(game_id)
        .bind(turn_time)
        .fetch_all(&self.pool)
        .await?)
    }

    pub async fn add_entity_for_player(
        &self,
        entity: &ShortEntity,
        game_id: &i64,
        seat_id: &i64,
    ) -> Result<i64, sqlx::error::Error> {
        Ok(sqlx::query(
            r#"
        INSERT INTO entity (game_id,
                seat_id,
                x,
                y,
                last_move_time,
                next_move_time,
                game_class) 
            VALUES (?, ?, ?, ?, 0, 0, ?)
            RETURNING id"#,
        )
        .bind(game_id)
        .bind(seat_id)
        .bind(entity.x)
        .bind(entity.y)
        .bind(entity.game_class.clone())
        .fetch_one(&self.pool)
        .await?
        .get(0))
    }

    pub async fn add_seat(
        &self,
        user_id: &str,
        game_id: &i64,
        scenario_player_id: &i64,
    ) -> Result<i64, sqlx::error::Error> {
        Ok(sqlx::query(
            r#"
        INSERT INTO seated_player (game_id,
                user_id,
                scenario_player_id) 
            VALUES (?, ?, ?) RETURNING id"#,
        )
        .bind(game_id)
        .bind(user_id)
        .bind(scenario_player_id)
        .fetch_one(&self.pool)
        .await?
        .get(0))
    }

    pub async fn start_game(&self, game_id: &i64) -> Result<(), sqlx::error::Error> {
        sqlx::query(
            r#"
        UPDATE game SET game_status = 'Running' WHERE id = ?; 
        "#,
        )
        .bind(game_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn count_seats_for_game(&self, game_id: &i64) -> Result<i64, sqlx::error::Error> {
        Ok(sqlx::query(
            r#"
        SELECT COUNT(*) FROM seated_player WHERE game_id = ?;
        "#,
        )
        .bind(game_id)
        .fetch_one(&self.pool)
        .await?
        .get(0))
    }

    pub async fn count_scenario_players_for_game(
        &self,
        game_id: &i64,
    ) -> Result<i64, sqlx::error::Error> {
        Ok(sqlx::query(
            r#"
        SELECT COUNT(*) FROM game JOIN scenario_player ON scenario_player.scenario_id = game.scenario_id WHERE game.id = ?;
        "#,
        )
        .bind(game_id)
        .fetch_one(&self.pool)
        .await?
        .get(0))
    }

    pub async fn get_trait_entity(&self, game_id: &i64) -> Result<Entity, sqlx::error::Error> {
        Ok(sqlx::query_as::<_, Entity>(
            r#"
        SELECT entity.*, seated_player.scenario_player_id, seated_player.user_id FROM entity JOIN seated_player ON entity.seat_id = seated_player.id WHERE entity.game_id = ? ORDER BY next_move_time, id LIMIT 1
        "#,
        )
        .bind(game_id)
        .fetch_one(&self.pool)
        .await?)
    }
    pub async fn get_entity(
        &self,
        game_id: &i64,
        entity_id: &i64,
    ) -> Result<Entity, sqlx::error::Error> {
        Ok(sqlx::query_as::<_, Entity>(
            r#"
        SELECT entity.*, seated_player.scenario_player_id, seated_player.user_id FROM entity JOIN seated_player ON entity.seat_id = seated_player.id WHERE entity.game_id = ? AND id = ?
        "#,
        )
        .bind(game_id)
        .bind(entity_id)
        .fetch_one(&self.pool)
        .await?)
    }

    pub async fn get_entity_at(
        &self,
        game_id: &i64,
        x: &i64,
        y: &i64,
    ) -> Result<Option<Entity>, sqlx::error::Error> {
        Ok(sqlx::query_as::<_, Entity>(
            r#"
        SELECT entity.*, seated_player.scenario_player_id, seated_player.user_id FROM entity JOIN seated_player
           ON entity.seat_id = seated_player.id WHERE entity.game_id = ? AND x = ? AND y = ?
        "#,
        )
        .bind(game_id)
        .bind(x)
        .bind(y)
        .fetch_optional(&self.pool)
        .await?)
    }

    pub async fn get_tile_at(
        &self,
        game_id: &i64,
        x: &i64,
        y: &i64,
    ) -> Result<Tile, sqlx::error::Error> {
        Ok(Tile::from(sqlx::query_as::<_, MapTile>(
            r#"
        SELECT tile.* FROM game JOIN scenario ON game.scenario_id = scenario.id JOIN tile on tile.map_id = scenario.map_id WHERE game.id = ? AND x = ? AND y = ?
        "#,
        )
        .bind(game_id)
        .bind(x)
        .bind(y)
        .fetch_one(&self.pool)
        .await?))
    }

    pub async fn set_entity(
        &self,
        game_id: &i64,
        entity: &Entity,
    ) -> Result<(), sqlx::error::Error> {
        sqlx::query(
            r#"
        UPDATE entity SET next_move_time = ?, last_move_time = ?, x = ?, y = ? WHERE game_id = ? AND id = ?
        "#,
        )
        .bind(entity.next_move_time)
        .bind(entity.last_move_time)
        .bind(entity.x)
        .bind(entity.y)
        .bind(game_id)
        .bind(entity.id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn transfer_entity(
        &self,
        game_id: &i64,
        entity_id: &i64,
        user_to: &str,
    ) -> Result<(), sqlx::error::Error> {
        // FIXME
        sqlx::query(
            r#"
        UPDATE entity SET user_id = ? WHERE game_id = ? AND entity_id = ?
        "#,
        )
        .bind(user_to)
        .bind(game_id)
        .bind(entity_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_entity_last_ability_name(
        &self,
        game_id: &i64,
        entity_id: &i64,
    ) -> Result<AbilityName, sqlx::error::Error> {
        Ok(AbilityName::from_str(sqlx::query(
                r#"
            SELECT action_name FROM action_log JOIN entity ON entity.id = caster WHERE entity.game_id = ? AND caster = ? AND action_log.turn_time = entity.last_move_time
            "#,
            )
            .bind(game_id)
            .bind(entity_id)
            .fetch_one(&self.pool)
            .await?.get::<&str, _>(0)).unwrap())
    }

    pub async fn get_game_entities(
        &self,
        game_id: &i64,
    ) -> Result<Vec<EntityWithResources>, sqlx::error::Error> {
        let entities = sqlx::query_as::<_, Entity>(r#"SELECT entity.*, seated_player.scenario_player_id, seated_player.user_id FROM entity JOIN seated_player ON entity.seat_id = seated_player.id WHERE entity.game_id = ?"#)
            .bind(game_id)
            .fetch_all(&self.pool)
            .await?;
        let mut entities_with_resources = Vec::new();
        for entity in entities {
            entities_with_resources.push(EntityWithResources {
                entity: entity.clone(),
                resources: self.get_entity_resources(game_id, &entity.id).await?,
            })
        }
        Ok(entities_with_resources)
    }

    pub async fn get_game_tiles(&self, game_id: &i64) -> Result<Vec<Tile>, sqlx::error::Error> {
        let tiles = sqlx::query_as::<_, MapTile>(
            r#"
        SELECT x, y, tile_type FROM game JOIN scenario ON game.scenario_id = scenario.id JOIN tile ON tile.map_id = scenario.map_id WHERE game.id = ?
        "#,
        )
        .bind(game_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(tiles.into_iter().map(|t| Tile::from(t)).collect())
    }

    pub async fn check_user(&self, username: &str, password: &str) -> Result<Vec<String>, String> {
        let result = sqlx::query_as::<_, UserData>(
            r#"
            SELECT roles, passhash FROM user WHERE username = ?
            "#,
        )
        .bind(username)
        .fetch_one(&self.pool)
        .await;
        match result {
            Err(error) => return Err(error.to_string()),
            Ok(user_data) => {
                tracing::info!(user_data.passhash);
                let parsed_hash = argon2::PasswordHash::new(&user_data.passhash).unwrap();
                match Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
                    Ok(_) => Ok(user_data
                        .roles
                        .split(",")
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>()),
                    _ => Err("Authentication failed".to_owned()),
                }
            }
        }
    }

    pub async fn register_user(&self, username: String, password: String) -> Result<(), String> {
        let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);

        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(&password.as_bytes(), &salt)
            .unwrap()
            .to_string();

        let result = sqlx::query(
            r#"
                INSERT INTO user (username, passhash, roles) VALUES (?, ?, "user");
                "#,
        )
        .bind(username)
        .bind(password_hash)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(error) => Err(error.to_string()),
        }
    }
}
