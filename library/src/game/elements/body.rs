use serde::{Deserialize, Serialize};

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
}

impl Body {
    pub fn new(body_type: BodyType) -> Body {
        Self { body_type }
    }
}

impl Body {
    // let body_type: i32 = row
    //     .try_get("body_type")
    //     .map_err(|err| Error::DbLoadSystemsError(err))?;
}
