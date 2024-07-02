use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{
    charclasses::CharClass,
    schemas::{Coords, Entity},
    services::get_distance,
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

pub struct Ability {
    pub name: AbilityName,
    pub caster: Entity,
}

impl Ability {
    pub fn min_range(&self) -> f64 {
        0.0
    }

    pub fn get_delay(&self, target: Coords) -> i64 {
        match self.name {
            AbilityName::ShieldBash | AbilityName::Attack => {
                let discount = match self.caster.game_class {
                    CharClass::Archer => {
                        let last_move = self.caster.log.last().map(|l| l.action_name.clone());
                        match last_move {
                            Some(AbilityName::Move) => 6,
                            _ => 0,
                        }
                    }
                    _ => 0,
                };
                20 - discount
            }
            AbilityName::Move => {
                let discount = match self.caster.game_class {
                    CharClass::Archer => {
                        let last_move = self.caster.log.last().map(|l| l.action_name.clone());
                        match last_move {
                            Some(AbilityName::Attack) => 6,
                            _ => 0,
                        }
                    }
                    _ => 0,
                };
                (12.0 * get_distance(&self.caster.coords, &target)) as i64 - discount
            }
            AbilityName::Wait => 6,
        }
    }

    pub fn max_range(&self, caster_class: &CharClass) -> f64 {
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

    pub fn get_costs(&self) -> Vec<(String, i64)> {
        match self.name {
            AbilityName::ShieldBash => vec![("ShieldBash".to_string(), 60)],
            _ => Vec::new(),
        }
    }
}
