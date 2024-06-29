use core::str::FromStr;

use crate::abilities::AbilityName;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum CharClass {
    Warrior,
    Archer,
    Mage,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ResourceStat {
    pub resource_name: String,
    pub resource_max: f64,
    pub resource_current: f64,
    pub resource_per_turn: f64,
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
    pub fn get_attack_damage(&self) -> f64 {
        match self {
            Self::Warrior => 14.0,
            Self::Archer => 12.0,
            Self::Mage => 10.0,
        }
    }
    pub fn get_attack_time(&self) -> i64 {
        return 20;
    }

    pub fn get_resource_list(&self) -> Vec<ResourceStat> {
        let mut resources = vec![ResourceStat {
            resource_name: "hp".to_string(),
            resource_max: 100.0,
            resource_current: 100.0,
            resource_per_turn: 0.5,
        }];

        match self {
            CharClass::Warrior => resources.push(ResourceStat {
                resource_name: "ShieldBash".to_string(),
                resource_max: 60.0,
                resource_current: 60.0,
                resource_per_turn: 1.0,
            }),
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
