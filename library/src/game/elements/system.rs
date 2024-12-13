use crate::error::Error;
use crate::game::element::Element;
use crate::game::instance::ElementContainer;
use crate::game::repr::{Angle, Distance, GalacticCoords};
use crate::Result;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqliteRow;
use sqlx::Row;
use std::str::FromStr;
use uuid::Uuid;

use super::move_from_local_delta;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
pub enum CenterType {
    OneStar,
    TwoStars,
    ThreeStars,
    BlackHole,
    NeutronStar,
}

impl Default for CenterType {
    fn default() -> Self {
        CenterType::OneStar
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct System {}

impl System {
    pub fn new() -> Self {
        Self {}
    }
}

impl System {
    // fn move_local(&mut self, delta: &crate::game::repr::SystemCoords) {
    //     self.coords = move_from_local_delta(&self.coords, delta);
    // }

    pub fn from_sqlite_row(row: &SqliteRow) -> Result<ElementContainer> {
        let uuid_str: &str = row
            .try_get("uuid")
            .map_err(|err| Error::DbLoadSystemsError(err))?;

        let uuid = Uuid::from_str(uuid_str).map_err(|err| Error::DbInvalidUuidError(err))?;

        let angle_1: Angle = row
            .try_get("phi")
            .map_err(|err| Error::DbLoadSystemsError(err))?;

        let angle_2: Angle = row
            .try_get("theta")
            .map_err(|err| Error::DbLoadSystemsError(err))?;

        let distance: Distance = row
            .try_get("distance")
            .map_err(|err| Error::DbLoadSystemsError(err))?;

        Ok(ElementContainer::new(
            Element::System(System::new()),
            uuid,
            GalacticCoords::new(angle_1, angle_2, distance),
        ))
    }

    fn update(&mut self, _delta: f32) -> bool {
        true
    }
}
