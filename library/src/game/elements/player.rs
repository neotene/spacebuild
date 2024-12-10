use crate::error::Error;
use crate::game::element::Element;
use crate::game::repr::{Angle, Coords, Direction, Distance};
use crate::protocol::PlayerAction;
use crate::Result;
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqliteRow;
use sqlx::Row;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct Player {
    pub(crate) uuid: Uuid,
    #[serde(skip_serializing)]
    pub(crate) synced: bool,
    pub(crate) coords: Coords,
    pub(crate) direction: Vector3<f64>,
    pub(crate) speed: f64,
    pub(crate) nickname: String,
    pub(crate) own_system_uuid: Uuid,
    pub(crate) current_system_uuid: Uuid,
    #[serde(skip_serializing)]
    pub(crate) actions: Vec<PlayerAction>,
}

impl Player {
    pub fn new(coords: Coords, nickname: String, system_uuid: Uuid) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            synced: false,
            coords,
            nickname,
            own_system_uuid: system_uuid,
            current_system_uuid: system_uuid,
            actions: Vec::new(),
            direction: Vector3::new(0., 0., 1.),
            speed: 0.,
        }
    }

    pub fn direction(&self) -> Direction {
        self.direction
    }

    pub fn speed(&self) -> f64 {
        self.speed
    }

    pub fn nickname(&self) -> &str {
        &self.nickname
    }

    pub fn own_system_uuid(&self) -> Uuid {
        self.own_system_uuid
    }

    pub fn current_system_uuid(&self) -> Uuid {
        self.current_system_uuid
    }
}

impl Element for Player {
    fn get_coords(&self) -> &Coords {
        &self.coords
    }

    fn from_sqlite_row(row: &SqliteRow) -> Result<impl Element> {
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

        let direction_x: f64 = row
            .try_get("dir_x")
            .map_err(|err| Error::DbLoadSystemsError(err))?;

        let direction_y: f64 = row
            .try_get("dir_y")
            .map_err(|err| Error::DbLoadSystemsError(err))?;

        let direction_z: f64 = row
            .try_get("dir_z")
            .map_err(|err| Error::DbLoadSystemsError(err))?;

        let speed: f64 = row
            .try_get("speed")
            .map_err(|err| Error::DbLoadSystemsError(err))?;

        let nickname: String = row
            .try_get("nickname")
            .map_err(|err| Error::DbLoadSystemsError(err))?;

        let own_system_uuid_str: String = row
            .try_get("own_system")
            .map_err(|err| Error::DbLoadSystemsError(err))?;

        let own_system_uuid = Uuid::from_str(own_system_uuid_str.as_str())
            .map_err(|err| Error::DbInvalidUuidError(err))?;

        let current_system_uuid_str: String = row
            .try_get("current_system")
            .map_err(|err| Error::DbLoadSystemsError(err))?;

        let current_system_uuid = Uuid::from_str(current_system_uuid_str.as_str())
            .map_err(|err| Error::DbInvalidUuidError(err))?;

        Ok(Player {
            coords: Coords::new(angle_1, angle_2, distance),
            direction: Vector3::new(direction_x, direction_y, direction_z),
            actions: Vec::new(),
            current_system_uuid,
            own_system_uuid,
            nickname: nickname,
            speed,
            synced: false,
            uuid,
        })
    }

    fn update(&mut self, _delta: f32) -> bool {
        for action in &self.actions {
            match action {
                // PlayerAction::ShipState(ship_state) => {
                //     self.direction.x = ship_state.direction.x;
                //     self.direction.y = ship_state.direction.y;
                //     self.direction.z = ship_state.direction.z;
                //     if ship_state.throttle_up {
                //         self.coords += self.direction * self.speed * delta;
                //     }
                // }
                _ => {
                    todo!();
                }
            }
        }
        self.actions.clear();
        true
    }

    fn get_sql_insert_line(&self) -> String {
        format!(
            "('{}', {}, {}, {}, {}, {}, {}, {}, '{}', '{}', '{}'),",
            self.uuid.to_string(),
            self.coords.angle_1,
            self.coords.angle_2,
            self.coords.distance,
            self.direction.x,
            self.direction.y,
            self.direction.z,
            self.speed,
            self.nickname,
            self.own_system_uuid,
            self.current_system_uuid,
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
