use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{
    charclasses::CharClass,
    schemas::{Coords, Entity},
    services::{get_distance, ServiceError},
    stores::database::Repository,
};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum AbilityName {
    ShieldBash,
    Move,
    Attack,
    Wait,
}

impl ToString for AbilityName {
    fn to_string(&self) -> String {
        match self {
            AbilityName::ShieldBash => "ShieldBash",
            AbilityName::Move => "Move",
            AbilityName::Attack => "Attack",
            AbilityName::Wait => "Wait",
        }
        .to_string()
    }
}

impl FromStr for AbilityName {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Move" => Ok(Self::Move),
            "Wait" => Ok(Self::Wait),
            "ShieldBash" => Ok(Self::ShieldBash),
            "Attack" => Ok(Self::Attack),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum TargetType {
    Walkable,
    Ennemy,
    Ally,
    Selfcast,
}

pub struct Ability<'a> {
    pub name: AbilityName,
    pub caster: &'a mut Entity,
}

impl Ability<'_> {
    pub async fn _take_damage(
        &self,
        repo: &Repository,
        game_id: &i64,
        damage: &f64,
        target_entity: &Entity,
    ) -> Result<(), ServiceError> {
        let mut hp = repo.get_resource(&game_id, &target_entity.id, "hp").await?;
        hp.resource_current -= damage;
        if hp.resource_current <= 0.0 {
            repo.remove_entity(&game_id, &target_entity.id).await?;
            let user_count = repo.count_active_players(&game_id).await?;
            if user_count <= 1 {
                repo.delete_game(&game_id).await?;
            }
        } else {
            repo.set_resource(&hp).await?;
        }
        Ok(())
    }

    pub async fn apply(
        &mut self,
        repo: &Repository,
        game_id: i64,
        target: Coords,
        target_entity: Option<Entity>,
    ) -> Result<(), ServiceError> {
        match self.name {
            AbilityName::ShieldBash => {
                let mut target_entity = target_entity.unwrap();
                self._take_damage(
                    &repo,
                    &game_id,
                    &(self.caster.get_class().get_attack_damage() * 0.8),
                    &target_entity,
                )
                .await?;
                target_entity.next_move_time += 12;
            }
            AbilityName::Move => {
                self.caster.x = target.x;
                self.caster.y = target.y;
                return Ok(());
            }
            AbilityName::Attack => {
                let target_entity = target_entity.unwrap();
                self._take_damage(
                    &repo,
                    &game_id,
                    &self.caster.get_class().get_attack_damage(),
                    &target_entity,
                )
                .await?;
            }
            AbilityName::Wait => {}
        }
        Ok(())
    }

    pub fn min_range(&self) -> f64 {
        0.0
    }

    pub async fn get_delay(&self, repo: &Repository, game_id: &i64, target: Coords) -> i64 {
        match self.name {
            AbilityName::ShieldBash | AbilityName::Attack => {
                let discount = match self.caster.get_class() {
                    CharClass::Archer => {
                        let last_move = repo
                            .get_entity_last_ability_name(&game_id, &self.caster.id)
                            .await;
                        match last_move {
                            Ok(AbilityName::Move) => 6,
                            _ => 0,
                        }
                    }
                    _ => 0,
                };
                20 - discount
            }
            AbilityName::Move => {
                let discount = match self.caster.get_class() {
                    CharClass::Archer => {
                        let last_move = repo
                            .get_entity_last_ability_name(&game_id, &self.caster.id)
                            .await;
                        match last_move {
                            Ok(AbilityName::Attack) => 6,
                            _ => 0,
                        }
                    }
                    _ => 0,
                };
                (12.0 * get_distance(self.caster.x, self.caster.y, target.x, target.y)) as i64
                    - discount
            }
            AbilityName::Wait => 6,
        }
    }

    pub fn max_range(&self, caster_class: CharClass) -> f64 {
        match self.name {
            AbilityName::ShieldBash | AbilityName::Attack => caster_class.get_attack_range(),
            AbilityName::Move => 2.0,
            AbilityName::Wait => 0.0,
        }
    }

    pub fn target_type(&self) -> TargetType {
        match self.name {
            AbilityName::ShieldBash | AbilityName::Attack => TargetType::Ennemy,
            AbilityName::Move => TargetType::Walkable,
            AbilityName::Wait => TargetType::Selfcast,
        }
    }

    pub fn needs_los(&self) -> bool {
        self.target_type() != TargetType::Selfcast
    }

    pub fn get_costs(&self) -> Vec<(String, f64)> {
        match self.name {
            AbilityName::ShieldBash => vec![("ShieldBash".to_string(), 60.0)],
            _ => Vec::new(),
        }
    }
}
