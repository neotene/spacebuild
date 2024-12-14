use super::element::{Body, Element};
use super::elements::body::BodyType;
use super::elements::player::Player;
use super::elements::system::System;
use super::repr::{Angle, Distance, GlobalCoords, LocalCoords, Speed};
use crate::error::Error;
use crate::Result;
use rand::Rng;
use sqlx::sqlite::SqliteRow;
use sqlx::Row;
use std::f64::consts::PI;
use std::str::FromStr;
use uuid::Uuid;

pub fn gen_system() -> (Galactic, Vec<Galactic>) {
    let mut rng = rand::thread_rng();
    let angle_1 = rng.gen_range(0.0..PI);
    let angle_2 = rng.gen_range(0.0..PI);
    let distance = rng.gen_range(0.0..100000.);

    let uuid = Uuid::new_v4();
    let coords = GlobalCoords::new(angle_1, angle_2, distance);

    let system = Galactic::new(
        Element::System(System::default()),
        uuid,
        coords.clone(),
        LocalCoords::default(),
        0.,
    );

    let mut bodies_in_system = Vec::<Galactic>::default();

    for _ in 1..100 {
        let x = rng.gen_range(0.0..1000.);
        let y = rng.gen_range(0.0..1000.);
        let z = rng.gen_range(0.0..10.);

        let mut cln = coords.clone();

        cln.translate_from_local_delta(&LocalCoords::new(x, y, z));
        bodies_in_system.push(Galactic::new(
            Element::Body(Body::new(BodyType::Asteroid, uuid)),
            Uuid::new_v4(),
            cln,
            LocalCoords::default(),
            0.,
        ));
    }

    (system, bodies_in_system)
}

type Container = Vec<Galactic>;

#[derive(Default, Clone)]
pub struct Galaxy {
    pub(crate) galactics: Container,
}

impl Galaxy {
    pub async fn update(&mut self, delta: f64) {
        let cln = self.clone();

        for galactic in &mut self.galactics {
            match &mut galactic.element {
                Element::Player(player) => {
                    let others = cln.borrow_bodies_of_system(player.current_system_uuid);
                    galactic.coords = player.update(
                        delta,
                        galactic.coords.clone(),
                        galactic.direction,
                        galactic.speed,
                        cln.borrow_galactic(player.current_system_uuid)
                            .unwrap()
                            .clone(),
                        others,
                    );
                }
                Element::Body(_body) => {
                    galactic.coords.translate_from_local_delta(
                        &(&galactic.direction * galactic.speed * delta),
                    );
                }
                Element::System(_system) => {}
            }
        }
    }

    pub fn borrow_bodies_of_system(&self, system_uuid: Uuid) -> Vec<&Galactic> {
        let mut collection = Vec::default();
        let bodies = self.borrow_bodies();

        for galactic in bodies {
            if let Element::Body(body) = &galactic.element {
                if body.owner_system_id == system_uuid {
                    collection.push(galactic);
                }
            } else {
                unreachable!()
            }
        }

        collection
    }

    pub fn borrow_systems(&self) -> Vec<&Galactic> {
        let mut collection = Vec::default();

        for element in &self.galactics {
            if let Element::System(_) = element.element {
                collection.push(element);
            }
        }

        collection
    }

    pub fn borrow_bodies(&self) -> Vec<&Galactic> {
        let mut collection = Vec::default();

        for element in &self.galactics {
            if let Element::Body(_) = element.element {
                collection.push(element);
            }
        }

        collection
    }

    pub fn borrow_players(&self) -> Vec<&Galactic> {
        let mut collection = Vec::default();

        for element in &self.galactics {
            if let Element::Player(_) = element.element {
                collection.push(element);
            }
        }

        collection
    }

    pub fn borrow_players_mut(&mut self) -> Vec<&mut Galactic> {
        let mut collection = Vec::default();

        for element in &mut self.galactics {
            if let Element::Player(_) = element.element {
                collection.push(element);
            }
        }

        collection
    }

    pub fn add_galactic(&mut self, galactic: Element, coords: GlobalCoords) -> Uuid {
        let uuid = Uuid::new_v4();
        self.galactics.push(Galactic::new(
            galactic,
            uuid,
            coords,
            LocalCoords::default(),
            0.,
        ));
        uuid
    }

    pub fn borrow_galactic(&self, uuid: Uuid) -> Option<&Galactic> {
        for galactic in &self.galactics {
            if galactic.uuid == uuid {
                return Some(galactic);
            }
        }
        None
    }

    pub fn borrow_galactic_mut(&mut self, uuid: Uuid) -> Option<&mut Galactic> {
        for galactic in &mut self.galactics {
            if galactic.uuid == uuid {
                return Some(galactic);
            }
        }
        None
    }
}

#[derive(Clone)]
pub struct Galactic {
    pub(crate) element: Element,
    pub(crate) uuid: Uuid,
    pub(crate) coords: GlobalCoords,
    pub(crate) direction: LocalCoords,
    pub(crate) speed: Speed,
}

impl Galactic {
    pub fn new(
        element: Element,
        uuid: Uuid,
        coords: GlobalCoords,
        direction: LocalCoords,
        speed: Speed,
    ) -> Self {
        Self {
            coords,
            direction,
            element,
            speed,
            uuid,
        }
    }

    pub fn get_uuid(&self) -> Uuid {
        self.uuid
    }

    pub fn get_coords(&self) -> GlobalCoords {
        self.coords.clone()
    }

    pub fn get_direction(&self) -> LocalCoords {
        self.direction
    }

    pub fn get_speed(&self) -> Speed {
        self.speed
    }

    pub fn get_element(&self) -> &Element {
        &self.element
    }

    pub fn get_sql_insert_line(&self) -> String {
        let mut sql_insert_line = format!(
            "('{}', {}, {}, {}, {}",
            self.uuid, self.coords.phi, self.coords.theta, self.coords.distance, self.speed
        );
        match &self.element {
            Element::System(_) => {}
            Element::Body(body) => {
                sql_insert_line += format!(
                    ", {}, {}, {}, {}, '{}'",
                    self.direction.x,
                    self.direction.y,
                    self.direction.z,
                    body.body_type as u32,
                    body.owner_system_id
                )
                .as_str()
            }
            Element::Player(player) => {
                sql_insert_line += format!(
                    ", {}, {}, {}, '{}', '{}', '{}'",
                    self.direction.x,
                    self.direction.y,
                    self.direction.z,
                    player.nickname,
                    player.own_system_uuid,
                    player.current_system_uuid
                )
                .as_str()
            }
        }
        sql_insert_line + ")"
    }

    pub fn from_sqlite_row(row: &SqliteRow) -> Result<Self> {
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

        let speed: f64 = row
            .try_get("speed")
            .map_err(|err| Error::DbLoadSystemsError(err))?;

        let direction_x: sqlx::Result<f64> = row.try_get("dir_x");
        let nickname: sqlx::Result<&str> = row.try_get("nickname");
        let body_type: sqlx::Result<i32> = row.try_get("type");

        let element_container = if direction_x.is_ok() {
            let direction_y: f64 = row
                .try_get("dir_y")
                .map_err(|err| Error::DbLoadSystemsError(err))?;

            let direction_z: f64 = row
                .try_get("dir_z")
                .map_err(|err| Error::DbLoadSystemsError(err))?;

            if nickname.is_ok() {
                let own_system_uuid: &str = row
                    .try_get("own_system")
                    .map_err(|err| Error::DbLoadSystemsError(err))?;
                let current_system_uuid: &str = row
                    .try_get("current_system")
                    .map_err(|err| Error::DbLoadSystemsError(err))?;
                Galactic::new(
                    Element::Player(Player::new(
                        nickname.unwrap().to_string(),
                        Uuid::from_str(own_system_uuid)
                            .map_err(|err| Error::BadUuidError(err, own_system_uuid.to_string()))?,
                        Uuid::from_str(current_system_uuid)
                            .map_err(|err| Error::BadUuidError(err, own_system_uuid.to_string()))?,
                    )),
                    uuid,
                    GlobalCoords::new(phi, theta, distance),
                    LocalCoords::new(
                        direction_x.map_err(|err| Error::DbLoadSystemsError(err))?,
                        direction_y,
                        direction_z,
                    ),
                    speed,
                )
            } else if body_type.is_ok() {
                let system_owner_uuid: &str = row
                    .try_get("system_owner")
                    .map_err(|err| Error::DbLoadSystemsError(err))?;
                Galactic::new(
                    Element::Body(Body::new(
                        (body_type.map_err(|err| Error::DbLoadSystemsError(err))? as u32).into(),
                        Uuid::from_str(system_owner_uuid).unwrap(),
                    )),
                    uuid,
                    GlobalCoords::new(phi, theta, distance),
                    LocalCoords::new(
                        direction_x.map_err(|err| Error::DbLoadSystemsError(err))?,
                        direction_y,
                        direction_z,
                    ),
                    speed,
                )
            } else {
                unreachable!()
            }
        } else {
            Galactic::new(
                Element::System(System::default()),
                uuid,
                GlobalCoords::new(phi, theta, distance),
                LocalCoords::default(),
                speed,
            )
        };
        Ok(element_container)
    }
}
