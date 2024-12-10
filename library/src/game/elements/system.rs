use crate::error::Error;
use crate::game::element::Element;
use crate::game::repr::{Angle, Coords, Distance};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqliteRow;
use sqlx::Row;
use std::str::FromStr;
use uuid::Uuid;

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

impl From<u32> for CenterType {
    fn from(value: u32) -> Self {
        match value {
            0 => CenterType::OneStar,
            1 => CenterType::TwoStars,
            2 => CenterType::ThreeStars,
            3 => CenterType::BlackHole,
            4 => CenterType::NeutronStar,
            _ => panic!("Invalid center type!"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct System {
    pub(crate) uuid: Uuid,
    pub(crate) synced: bool,
    pub(crate) coords: Coords,
    pub(crate) center_type: CenterType,
}

impl System {
    pub fn new(coords: Coords, center_type: CenterType) -> Self {
        Self {
            synced: false,
            uuid: Uuid::new_v4(),
            coords,
            center_type,
        }
    }

    pub fn get_center_type(&self) -> CenterType {
        self.center_type
    }
}

impl Element for System {
    fn from_sqlite_row(row: &SqliteRow) -> crate::Result<impl Element> {
        let uuid_str: &str = row
            .try_get("uuid")
            .map_err(|err| Error::DbLoadSystemsError(err))?;

        let uuid = Uuid::from_str(uuid_str).map_err(|err| Error::DbInvalidUuidError(err))?;

        let angle_1: Angle = row
            .try_get("angle_1")
            .map_err(|err| Error::DbLoadSystemsError(err))?;

        let angle_2: Angle = row
            .try_get("angle_2")
            .map_err(|err| Error::DbLoadSystemsError(err))?;

        let distance: Distance = row
            .try_get("distance")
            .map_err(|err| Error::DbLoadSystemsError(err))?;

        let center_type: CenterType = row
            .try_get::<u32, &str>("center_type")
            .map_err(|err| Error::DbLoadSystemsError(err))?
            .into();

        Ok(System {
            coords: Coords::new(angle_1, angle_2, distance),
            center_type,
            synced: false,
            uuid,
        })
    }

    fn get_coords(&self) -> &Coords {
        &self.coords
    }

    fn update(&mut self, _delta: f32) -> bool {
        true
    }

    fn get_sql_insert_line(&self) -> String {
        format!(
            "('{}', {}, {}, {}, {}),",
            self.uuid.to_string(),
            self.coords.angle_1,
            self.coords.angle_2,
            self.coords.distance,
            // self.speed,
            self.center_type as usize
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
