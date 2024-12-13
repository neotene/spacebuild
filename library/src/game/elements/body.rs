use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::repr::GalacticCoords;
use crate::Result;

use super::move_from_local_delta;

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
    pub coords: GalacticCoords,
    pub velocity: Vector3<f32>,
}

impl Body {
    pub fn new(body_type: BodyType, coords: GalacticCoords) -> Body {
        Self {
            uuid: Uuid::new_v4(),
            synced: false,
            body_type,
            coords,
            velocity: Vector3::default(),
        }
    }
}

impl Body {
    fn get_coords(&self) -> &GalacticCoords {
        &self.coords
    }

    fn from_sqlite_row(_row: &sqlx::sqlite::SqliteRow) -> Result<Body> {
        Ok(Body::new(
            BodyType::Asteroid,
            GalacticCoords::new(0., 0., 0.),
        ))
    }

    // fn get_global_coords(&self) -> &crate::game::repr::GalacticCoords {
    //     &self.coords
    // }

    // fn get_local_coords(&self) -> &crate::game::repr::SystemCoords {
    //     &self.local_coords
    // }

    // fn move_global(&mut self, delta: &GalacticCoords) {
    //     use scilib::coordinate::*;

    //     let car = cartesian::Cartesian::from(
    //         self.local_coords.x,
    //         self.local_coords.y,
    //         self.local_coords.z,
    //     );

    //     let sph = spherical::Spherical::from_degree(1.2, 30, 60.2);
    // }

    fn move_local(&mut self, delta: &crate::game::repr::SystemCoords) {
        self.coords = move_from_local_delta(&self.coords, delta);
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

    fn get_name(&self) -> String {
        self.uuid.to_string()
    }
}
