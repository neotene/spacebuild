use crate::error::Error;
use crate::game::element::Element;
use crate::game::instance::ElementContainer;
use crate::game::repr::{Angle, Direction, Distance, GalacticCoords};
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
    pub(crate) nickname: String,
    pub(crate) own_system_uuid: Uuid,
    pub(crate) current_system_uuid: Uuid,
    #[serde(skip_serializing)]
    pub(crate) actions: Vec<PlayerAction>,
}

impl Player {
    pub fn new(nickname: String, own_system_uuid: Uuid, current_system_uuid: Uuid) -> Self {
        Self {
            nickname,
            own_system_uuid,
            current_system_uuid,
            actions: Vec::default(),
        }
    }

    pub fn get_nickname(&self) -> &str {
        &self.nickname
    }

    pub fn own_system_uuid(&self) -> Uuid {
        self.own_system_uuid
    }

    pub fn current_system_uuid(&self) -> Uuid {
        self.current_system_uuid
    }
}

impl Player {
    pub fn from_sqlite_row(row: &SqliteRow) -> Result<ElementContainer> {
        let uuid_str: &str = row
            .try_get("uuid")
            .map_err(|err| Error::DbLoadSystemsError(err))?;

        let uuid = Uuid::from_str(uuid_str).map_err(|err| Error::DbInvalidUuidError(err))?;

        let phi: Angle = row
            .try_get("phi")
            .map_err(|err| Error::DbLoadSystemsError(err))?;

        let theta: Angle = row
            .try_get("theta")
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

        Ok(ElementContainer::new(
            Element::Player(Player::new(nickname, own_system_uuid, current_system_uuid)),
            Uuid::new_v4(),
            GalacticCoords::new(phi, theta, distance),
        ))
    }

    fn update(&mut self, _delta: f32) -> bool {
        for action in &self.actions {
            match action {
                PlayerAction::ShipState(ship_state) => {
                    if ship_state.throttle_up {
                        // self.coords += self.direction * self.speed * delta;
                    }
                }
                _ => {
                    todo!();
                }
            }
        }
        self.actions.clear();
        true
    }

    fn get_sql_insert_line(&self) -> String {
        format!("()")
    }

    fn get_name(&self) -> String {
        self.nickname.clone()
    }
}
