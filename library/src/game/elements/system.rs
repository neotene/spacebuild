use crate::error::Error;
use crate::game::element::Element;
use crate::game::repr::{Angle, Distance, GalacticCoords};
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
pub struct System {
    pub(crate) uuid: Uuid,
    pub(crate) synced: bool,
    pub(crate) coords: GalacticCoords,
}

impl System {
    pub fn new(coords: GalacticCoords) -> Self {
        Self {
            synced: false,
            uuid: Uuid::new_v4(),
            coords,
        }
    }
}

impl Element for System {
    fn get_coords(&self) -> &GalacticCoords {
        &self.coords
    }
    fn move_local(&mut self, delta: &crate::game::repr::SystemCoords) {
        self.coords = move_from_local_delta(&self.coords, delta);
    }

    fn from_sqlite_row(row: &SqliteRow) -> crate::Result<impl Element> {
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

        Ok(System {
            coords: GalacticCoords::new(angle_1, angle_2, distance),
            synced: false,
            uuid,
        })
    }

    fn update(&mut self, _delta: f32) -> bool {
        true
    }

    fn get_sql_insert_line(&self) -> String {
        format!(
            "('{}', {}, {}, {}),",
            self.uuid.to_string(),
            self.coords.theta,
            self.coords.phi,
            self.coords.distance,
        )
    }

    fn get_uuid(&self) -> Uuid {
        self.uuid
    }

    fn is_synced(&self) -> bool {
        self.synced
    }

    fn set_synced(&mut self, is_synced: bool) {
        self.synced = is_synced;
    }
}
