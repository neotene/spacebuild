use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::{element::Element, repr::Coords};

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
    pub uuid: Uuid,
    pub synced: bool,
    pub body_type: BodyType,
    pub coords: Coords,
    pub velocity: Vector3<f32>,
}

impl Body {
    pub fn new(body_type: BodyType, coords: Coords) -> Body {
        Self {
            uuid: Uuid::new_v4(),
            synced: false,
            body_type,
            coords,
            velocity: Vector3::default(),
        }
    }
}

impl Element for Body {
    fn from_sqlite_row(_row: &sqlx::sqlite::SqliteRow) -> crate::Result<impl Element> {
        Ok(Body::new(BodyType::Asteroid, Coords::new(0., 0., 0)))
    }

    fn get_coords(&self) -> &crate::game::repr::Coords {
        &self.coords
    }

    fn update(&mut self, _delta: f32) -> bool {
        // self.coords += self.velocity * delta;
        true
    }

    fn get_sql_insert_line(&self) -> String {
        assert!(false);
        format!("(),")
    }

    fn get_uuid(&self) -> uuid::Uuid {
        self.uuid
    }

    fn is_synced(&self) -> bool {
        self.synced
    }

    fn set_synced(&mut self, is_synced: bool) {
        self.synced = is_synced;
    }
}
