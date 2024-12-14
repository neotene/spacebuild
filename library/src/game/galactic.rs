use super::element::{Body, Element};
use super::elements::player::Player;
use super::elements::system::System;
use super::repr::{Angle, Distance, GalacticCoords, LocalCoords, Speed};
use crate::error::Error;
use crate::protocol::{ElementInfo, GameInfo, PlayerInfo};
use crate::Result;
use sqlx::sqlite::SqliteRow;
use sqlx::Row;
use std::str::FromStr;
use uuid::Uuid;

type Container = Vec<Galactic>;

#[derive(Default, Clone)]
pub struct Galaxy {
    pub(crate) galactics: Container,
}

impl Galaxy {
    pub async fn get_systems(&self) -> Vec<&Galactic> {
        let mut collection = Vec::default();

        for element in &self.galactics {
            if let Element::System(_) = element.element {
                collection.push(element);
            }
        }

        collection
    }

    pub async fn get_players(&self) -> Vec<&Galactic> {
        let mut collection = Vec::default();

        for element in &self.galactics {
            if let Element::Player(_) = element.element {
                collection.push(element);
            }
        }

        collection
    }

    pub fn add_galactic(&mut self, element: Element, coords: GalacticCoords) -> Uuid {
        let uuid = Uuid::new_v4();
        self.galactics.push(Galactic::new(
            element,
            uuid,
            coords,
            LocalCoords::default(),
            0.,
        ));
        uuid
    }

    pub async fn borrow_galactic(&self, uuid: Uuid) -> Option<&Galactic> {
        for element in &self.galactics {
            if element.uuid == uuid {
                return Some(element);
            }
        }
        None
    }

    pub async fn borrow_galactic_mut(&mut self, uuid: Uuid) -> Option<&mut Galactic> {
        for element in &mut self.galactics {
            if element.uuid == uuid {
                return Some(element);
            }
        }
        None
    }
}

#[derive(Clone)]
pub struct Galactic {
    pub(crate) element: Element,
    pub(crate) uuid: Uuid,
    pub(crate) coords: GalacticCoords,
    pub(crate) direction: LocalCoords,
    pub(crate) speed: Speed,
}

impl Galactic {
    pub fn new(
        element: Element,
        uuid: Uuid,
        coords: GalacticCoords,
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

    pub async fn update(&mut self, delta: f64, others: &Galaxy) {
        let direction = self.direction.normalize();
        let speed = self.speed;
        let mut new_coords = self.coords.clone();
        let element = &mut self.element;
        match element {
            Element::Body(_body) => {
                new_coords.translate_from_local_delta(&(&direction * speed * delta));
            }
            Element::Player(player) => {
                for action in &player.actions {
                    match action {
                        crate::protocol::PlayerAction::ShipState(ship_state) => {
                            if ship_state.throttle_up {
                                new_coords
                                    .translate_from_local_delta(&(&direction * speed * delta));
                            }
                        }
                        _ => {
                            unreachable!()
                        }
                    }
                }

                player.actions.clear();

                player.game_infos.push(GameInfo::Player(PlayerInfo {
                    coords: new_coords.get_local_from_element(
                        &others
                            .borrow_galactic(player.current_system_uuid)
                            .await
                            .unwrap(),
                    ),
                }));

                let player_current_system =
                    others.borrow_galactic(player.current_system_uuid).await;
                assert!(player_current_system.is_some());
                let uuid = player_current_system.unwrap().uuid;
                let mut elements_infos = Vec::<ElementInfo>::default();
                for element in &others.galactics {
                    if let Element::Body(body) = &element.element {
                        if body.owner_system_id == uuid {
                            let coords = new_coords.get_local_from_element(
                                &others
                                    .borrow_galactic(player.current_system_uuid)
                                    .await
                                    .unwrap(),
                            );
                            elements_infos.push(ElementInfo { coords });
                        }
                    }
                }
                if !elements_infos.is_empty() {
                    player
                        .game_infos
                        .push(GameInfo::ElementsInSystem(elements_infos));
                }
            }
            Element::System(_system) => {}
        };

        self.coords = new_coords;
    }

    pub fn get_uuid(&self) -> Uuid {
        self.uuid
    }

    pub fn get_coords(&self) -> GalacticCoords {
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
                    GalacticCoords::new(phi, theta, distance),
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
                    GalacticCoords::new(phi, theta, distance),
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
                GalacticCoords::new(phi, theta, distance),
                LocalCoords::default(),
                speed,
            )
        };
        Ok(element_container)
    }
}
