use super::element::{Body, Element};
use super::elements::player::{self, Player};
use super::elements::system::{self, System};
use super::repr::{Angle, Distance, GalacticCoords, Speed, SystemCoords};
use crate::error::Error;
use crate::protocol::{ElementInfo, GameInfo, MyVector3, PlayerInfo};
use crate::Result;
use futures::TryStreamExt;
use rand::Rng;
use sqlx::sqlite::SqliteRow;
use sqlx::Row;
use sqlx::{Pool, Sqlite, SqlitePool};
use std::borrow::Borrow;
use std::default;
use std::fs::File;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

type ElementsContainer = Vec<Arc<Mutex<ElementContainer>>>;
#[derive(Default)]
struct Galaxy {
    elements: ElementsContainer,
}

impl Galaxy {
    pub async fn get_systems(&self) -> ElementsContainer {
        let mut collection = ElementsContainer::default();

        for element in &self.elements {
            if let Element::System(_) = element.lock().await.element {
                collection.push(Arc::clone(&element));
            }
        }

        collection
    }

    pub async fn get_players(&self) -> ElementsContainer {
        let mut collection = ElementsContainer::default();

        for element in &self.elements {
            if let Element::Player(_) = element.lock().await.element {
                collection.push(Arc::clone(&element));
            }
        }

        collection
    }

    pub fn add_element(&mut self, element: Element, coords: GalacticCoords) -> Uuid {
        let uuid = Uuid::new_v4();
        self.elements
            .push(Arc::new(Mutex::new(ElementContainer::new(
                element,
                uuid,
                coords,
                SystemCoords::default(),
                0.,
            ))));
        uuid
    }

    pub async fn get_element(&self, uuid: Uuid) -> Option<Arc<Mutex<ElementContainer>>> {
        for element in &self.elements {
            if element.lock().await.uuid == uuid {
                return Some(Arc::clone(element));
            }
        }
        None
    }
}

pub struct ElementContainer {
    pub(crate) element: Element,
    pub(crate) uuid: Uuid,
    pub(crate) coords: GalacticCoords,
    pub(crate) direction: SystemCoords,
    pub(crate) speed: Speed,
}

impl ElementContainer {
    pub fn new(
        element: Element,
        uuid: Uuid,
        coords: GalacticCoords,
        direction: SystemCoords,
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
        match &mut self.element {
            Element::Body(_body) => {
                self.coords
                    .translate_from_local_delta(&(self.direction.normalize() * self.speed * delta));
            }
            Element::Player(player) => {
                self.coords
                    .translate_from_local_delta(&(self.direction.normalize() * self.speed * delta));
                player.game_infos.push(GameInfo::Player(PlayerInfo {
                    coords: self.coords.get_local_from_element(
                        others
                            .get_element(player.current_system_uuid)
                            .await
                            .unwrap()
                            .lock()
                            .await
                            .borrow(),
                    ),
                }));

                let player_current_system = others.get_element(player.current_system_uuid).await;
                assert!(player_current_system.is_some());
                let uuid = player_current_system.unwrap().lock().await.uuid;
                let mut elements_infos = Vec::<ElementInfo>::default();
                for element in &others.elements {
                    if let Element::Body(body) = &element.lock().await.deref().element {
                        if body.owner_system_id == uuid {
                            let coords = self.coords.get_local_from_element(
                                others
                                    .get_element(player.current_system_uuid)
                                    .await
                                    .unwrap()
                                    .lock()
                                    .await
                                    .borrow(),
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
        }
    }

    pub fn get_uuid(&self) -> Uuid {
        self.uuid
    }

    pub fn get_coords(&self) -> GalacticCoords {
        self.coords.clone()
    }

    pub fn get_direction(&self) -> SystemCoords {
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
                    ", {}, {}, {}, {}",
                    self.direction.x, self.direction.y, self.direction.z, body.body_type as u32
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
                ElementContainer::new(
                    Element::Player(Player::new(
                        nickname.unwrap().to_string(),
                        Uuid::from_str(own_system_uuid)
                            .map_err(|err| Error::BadUuidError(err, own_system_uuid.to_string()))?,
                        Uuid::from_str(current_system_uuid)
                            .map_err(|err| Error::BadUuidError(err, own_system_uuid.to_string()))?,
                    )),
                    uuid,
                    GalacticCoords::new(phi, theta, distance),
                    SystemCoords::new(
                        direction_x.map_err(|err| Error::DbLoadSystemsError(err))?,
                        direction_y,
                        direction_z,
                    ),
                    speed,
                )
            } else if body_type.is_ok() {
                ElementContainer::new(
                    Element::Body(Body::new(
                        (body_type.map_err(|err| Error::DbLoadSystemsError(err))? as u32).into(),
                    )),
                    uuid,
                    GalacticCoords::new(phi, theta, distance),
                    SystemCoords::new(
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
            ElementContainer::new(
                Element::System(System::default()),
                uuid,
                GalacticCoords::new(phi, theta, distance),
                SystemCoords::default(),
                speed,
            )
        };
        Ok(element_container)
    }
}

pub struct Instance {
    pub(crate) pool: Pool<Sqlite>,
    pub(crate) galaxy: Galaxy,
}

impl Instance {
    pub const CREATE_TABLE_PLAYER_SQL_STR: &str = "CREATE TABLE IF NOT EXISTS Player (uuid TEXT PRIMARY KEY, phi REAL, theta REAL, distance REAL, speed REAL, dir_x REAL, dir_y REAL, dir_z REAL, nickname TEXT, own_system TEXT, current_system TEXT)";

    pub const CREATE_TABLE_SYSTEM_SQL_STR: &str = "CREATE TABLE IF NOT EXISTS System (uuid TEXT PRIMARY KEY, phi REAL, theta REAL, distance REAL, speed REAL)";

    pub const CREATE_TABLE_BODY_SQL_STR: &str = "CREATE TABLE IF NOT EXISTS Body (uuid TEXT PRIMARY KEY, phi REAL, theta REAL, distance REAL, speed REAL, dir_x REAL, dir_y REAL, dir_z REAL, type INT, system_owner TEXT, FOREIGN KEY(system_owner) REFERENCES System(uuid))";

    pub async fn from_path(db_path: &'_ str) -> Result<Instance> {
        if !Path::new(db_path).exists() {
            File::create(db_path).map_err(|err| Error::DbFileCreationError(err))?;
        }

        let pool = SqlitePool::connect(db_path)
            .await
            .map_err(|err| Error::DbOpenError(db_path.to_string(), err))?;

        Self::init(&pool).await?;

        let mut instance = Instance {
            pool,
            galaxy: Galaxy::default(),
        };

        instance.load_systems().await?;

        Ok(instance)
    }

    async fn init(pool: &Pool<Sqlite>) -> Result<()> {
        sqlx::query(Self::CREATE_TABLE_PLAYER_SQL_STR)
            .execute(pool)
            .await
            .map_err(|err| Error::DbCreatePlayerTableError(err))?;

        sqlx::query(Self::CREATE_TABLE_SYSTEM_SQL_STR)
            .execute(pool)
            .await
            .map_err(|err| Error::DbCreatePlayerTableError(err))?;

        sqlx::query(Self::CREATE_TABLE_BODY_SQL_STR)
            .execute(pool)
            .await
            .map_err(|err| Error::DbCreatePlayerTableError(err))?;

        Ok(())
    }

    pub async fn sync_to_db(&mut self) -> Result<()> {
        let mut insert_systems_sql_str = "INSERT INTO System VALUES ".to_string();
        let mut insert_bodies_sql_str = "INSERT INTO Body VALUES ".to_string();
        let mut insert_players_sql_str = "INSERT INTO Player VALUES ".to_string();
        for element in &self.galaxy.elements {
            let guard = element.lock().await;
            match guard.element {
                Element::System(_) => {
                    insert_systems_sql_str += guard.get_sql_insert_line().as_str();
                    insert_systems_sql_str += ", ";
                }
                Element::Body(_) => {
                    insert_bodies_sql_str += guard.get_sql_insert_line().as_str();
                    insert_bodies_sql_str += ", ";
                }
                Element::Player(_) => {
                    insert_players_sql_str += guard.get_sql_insert_line().as_str();
                    insert_players_sql_str += ", ";
                }
            }
        }

        if let Some(sql_str) = insert_systems_sql_str.strip_suffix(", ") {
            let _ = sqlx::query(&sql_str)
                .fetch_all(&self.pool)
                .await
                .map_err(|err| Error::DbSyncToDbError(err))?;
        }

        if let Some(sql_str) = insert_bodies_sql_str.strip_suffix(", ") {
            let _ = sqlx::query(&sql_str)
                .fetch_all(&self.pool)
                .await
                .map_err(|err| Error::DbSyncToDbError(err))?;
        }

        if let Some(sql_str) = insert_players_sql_str.strip_suffix(", ") {
            let _ = sqlx::query(&sql_str)
                .fetch_all(&self.pool)
                .await
                .map_err(|err| Error::DbSyncToDbError(err))?;
        }

        Ok(())
    }

    pub fn get_galaxy(&self) -> &Galaxy {
        &self.galaxy
    }

    pub async fn load_systems(&mut self) -> Result<()> {
        let mut rows = sqlx::query("SELECT * FROM System").fetch(&self.pool);

        while let Some(row) = rows
            .try_next()
            .await
            .map_err(|err| Error::DbLoadSystemsError(err))?
        {
            self.galaxy
                .elements
                .push(Arc::new(Mutex::new(ElementContainer::from_sqlite_row(
                    &row,
                )?)));
        }

        Ok(())
    }

    pub async fn load_player_by_nickname(&mut self, nickname: String) -> Result<Uuid> {
        let rows = sqlx::query("SELECT * FROM Player WHERE nickname=?")
            .bind(&nickname)
            .fetch_all(&self.pool)
            .await
            .map_err(|err| Error::DbLoadPlayerByNicknameQueryError(err))?;

        if rows.len() == 0 {
            return Err(Error::DbLoadPlayerByNicknameNotFound);
        }

        if rows.len() > 1 {
            return Err(Error::DbLoadPlayerByNicknameFoundTooMany);
        }

        let first = rows.first().unwrap();

        let player = ElementContainer::from_sqlite_row(first)?;

        let uuid = player.uuid;

        for element in &self.galaxy.elements {
            if element.lock().await.uuid == uuid {
                return Err(Error::PlayerAlreadyAuthenticated);
            }
        }

        self.galaxy.elements.push(Arc::new(Mutex::new(player)));

        Ok(uuid)
    }

    pub async fn leave(&mut self, uuid: Uuid) {
        let mut i = 0;
        for element in &self.galaxy.elements {
            if element.lock().await.uuid == uuid {
                break;
            }
            i += 1;
        }

        assert!(i < self.galaxy.elements.len());

        self.galaxy.elements.remove(i);
    }

    pub async fn authenticate(instance: &mut Instance, nickname: &String) -> Result<Uuid> {
        let maybe_uuid = instance.load_player_by_nickname(nickname.clone()).await;

        match maybe_uuid {
            Err(Error::DbLoadPlayerByNicknameNotFound) => {
                let player_system = gen_system();
                let player_sys_uuid = player_system.uuid;
                instance
                    .galaxy
                    .elements
                    .push(Arc::new(Mutex::new(player_system)));

                let player = ElementContainer::new(
                    Element::Player(Player::new(
                        nickname.clone(),
                        player_sys_uuid,
                        player_sys_uuid,
                    )),
                    Uuid::new_v4(),
                    GalacticCoords::default(),
                    SystemCoords::default(),
                    0.,
                );
                let uuid = player.uuid;
                instance.galaxy.elements.push(Arc::new(Mutex::new(player)));
                instance.sync_to_db().await?;

                Ok(uuid)
            }
            Ok(uuid) => Ok(uuid),
            Err(err) => Err(err),
        }
    }

    pub async fn update(&mut self, delta: f64) -> bool {
        for element in &mut self.galaxy.elements {
            element.lock().await.deref_mut().update(delta, self.galaxy);
        }
        false
    }
}

pub fn gen_system() -> (ElementContainer, Vec<ElementContainer>) {
    let mut rng = rand::thread_rng();
    let angle_1 = rng.gen_range(0..15000) as f64 / 10000.;
    let angle_2 = rng.gen_range(0..15000) as f64 / 10000.;
    let distance = rng.gen_range(0.0..10000000000.);

    let bodies_in_system = Vec::<ElementContainer>::default();

    (
        ElementContainer::new(
            Element::System(System::default()),
            Uuid::new_v4(),
            GalacticCoords::new(angle_1, angle_2, distance),
            SystemCoords::default(),
            0.,
        ),
        bodies_in_system,
    )
}
