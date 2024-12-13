use std::str::FromStr;

use crate::error::Error;
use crate::game::element::Element;
use crate::game::instance::ElementContainer;
use crate::game::repr::{Angle, Distance, GalacticCoords};
use crate::Result;
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqliteRow;
use sqlx::Row;
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
}

impl Body {
    pub fn new(body_type: BodyType) -> Body {
        Self { body_type }
    }
}

impl Body {
    fn from_sqlite_row(row: &sqlx::sqlite::SqliteRow) -> Result<ElementContainer> {
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

        let body_type: i32 = row
            .try_get("body_type")
            .map_err(|err| Error::DbLoadSystemsError(err))?;

        Ok(ElementContainer::new(
            Element::Body(Body::new((body_type as u32).into())),
            uuid,
            GalacticCoords::new(phi, theta, distance),
        ))
    }

    fn get_sql_insert_line(&self) -> String {
        assert!(false);
        format!("(),")
    }
}
