use core::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{abilities::AbilityName, schemas::Resource};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub enum CharClass {
    Warrior,
    Archer,
    Mage,
}

impl FromStr for CharClass {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Warrior" => Ok(Self::Warrior),
            "Archer" => Ok(Self::Archer),
            _ => Err("No matching class".to_string()),
        }
    }
}

impl CharClass {
    pub fn get_attack_range(&self) -> f64 {
        match self {
            Self::Warrior => 1.0,
            Self::Archer => 4.0,
            Self::Mage => 3.0,
        }
    }
    pub fn get_attack_damage(&self) -> i64 {
        match self {
            Self::Warrior => 140,
            Self::Archer => 120,
            Self::Mage => 100,
        }
    }
    pub fn get_attack_time(&self) -> i64 {
        return 20;
    }

    pub fn get_resource_list(&self) -> Vec<(String, Resource)> {
        let mut resources = vec![(
            "hp".to_string(),
            Resource {
                max: 1000,
                current: 1000,
                per_turn: 1,
            },
        )];

        match self {
            CharClass::Warrior => resources.push((
                "ShieldBash".to_string(),
                Resource {
                    max: 60,
                    current: 60,
                    per_turn: 1,
                },
            )),
            _ => {}
        }
        resources
    }

    pub fn get_ability_list(&self) -> Vec<AbilityName> {
        let mut abilites = vec![AbilityName::Move, AbilityName::Attack, AbilityName::Wait];
        match self {
            Self::Warrior => abilites.push(AbilityName::ShieldBash),
            _ => {}
        };
        abilites
    }
}
