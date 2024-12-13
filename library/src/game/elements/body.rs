use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Copy)]
pub enum BodyType {
    Planet,
    Asteroid,
    Station,
}

impl From<u32> for BodyType {
    fn from(value: u32) -> Self {
        match value {
            0 => BodyType::Planet,
            1 => BodyType::Asteroid,
            2 => BodyType::Station,
            _ => panic!("Invalid body type!"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Body {
    pub body_type: BodyType,
    pub owner_system_id: Uuid,
}

impl Body {
    pub fn new(body_type: BodyType, owner_system_id: Uuid) -> Body {
        Self {
            body_type,
            owner_system_id,
        }
    }
}
