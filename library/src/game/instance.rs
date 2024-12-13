use crate::error::Error;
use crate::Result;
use futures::TryStreamExt;
use nalgebra::Vector3;
use rand::Rng;
use regex::Regex;
use sqlx::{Pool, Sqlite, SqlitePool};
use std::fs::File;
use std::ops::Deref;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use super::element::Element;
use super::elements::player::Player;
use super::elements::system::System;
use super::repr::{GalacticCoords, SystemCoords};

pub struct ElementContainer {
    pub(crate) element: Element,
    pub(crate) uuid: Uuid,
    pub(crate) synced: bool,
    pub(crate) coords: GalacticCoords,
    pub(crate) direction: Vector3<f64>,
    pub(crate) speed: f64,
}

impl ElementContainer {
    pub fn new(element: Element, uuid: Uuid, coords: GalacticCoords) -> Self {
        Self {
            coords,
            direction: SystemCoords::default(),
            element,
            speed: 0.,
            synced: false,
            uuid,
        }
    }
}

type ElementsCollection = Vec<Arc<Mutex<ElementContainer>>>;

pub struct Instance {
    pub(crate) pool: Pool<Sqlite>,
    pub(crate) elements: ElementsCollection,
}

impl Instance {
    pub const CREATE_TABLE_PLAYER_SQL_STR: &str = "CREATE TABLE IF NOT EXISTS Player (uuid TEXT PRIMARY KEY, phi REAL, theta REAL, distance REAL, dir_x REAL, dir_y REAL, dir_z REAL, speed REAL, nickname TEXT, own_system TEXT, current_system TEXT)";

    pub const CREATE_TABLE_SYSTEM_SQL_STR: &str = "CREATE TABLE IF NOT EXISTS System (uuid TEXT PRIMARY KEY, phi REAL, theta REAL, distance REAL)";

    pub const CREATE_TABLE_BODY_SQL_STR: &str = "CREATE TABLE IF NOT EXISTS Body (uuid TEXT PRIMARY KEY, phi REAL, theta REAL, distance REAL, speed REAL, type INT, system_owner TEXT, FOREIGN KEY(system_owner) REFERENCES System(uuid))";

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
            elements: Vec::new(),
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
        Ok(())
    }

    const REGEX_STR: &str = "^(.*::)([^:]+)$";

    fn get_insert_query<T>() -> String {
        let regex = Regex::new(Self::REGEX_STR).unwrap();
        let full_type_name = std::any::type_name::<T>();

        let mut results = vec![];

        for (_, [prefix_type, short_type]) in
            regex.captures_iter(&full_type_name).map(|c| c.extract())
        {
            results.push((prefix_type, short_type));
        }

        "INSERT INTO ".to_string() + results.last().unwrap().1 + " VALUES "
    }

    // pub fn get_elements(&self) -> &ElementsCollection {
    //     &self.elements
    // }

    // pub fn add_system(&mut self, system: System, coords: GalacticCoords) {
    //     self.elements
    //         .push(Arc::new(Mutex::new(ElementContainer::new(
    //             Element::System(system),
    //             Uuid::new_v4(),
    //             coords,
    //         ))));
    // }

    // pub fn add_player(&mut self, player: Player, coords: GalacticCoords) {
    //     self.elements
    //         .push(Arc::new(Mutex::new(ElementContainer::new(
    //             Element::Player(player),
    //             Uuid::new_v4(),
    //             coords,
    //         ))));
    // }

    pub async fn load_systems(&mut self) -> Result<()> {
        let mut rows = sqlx::query("SELECT * FROM System").fetch(&self.pool);

        while let Some(row) = rows
            .try_next()
            .await
            .map_err(|err| Error::DbLoadSystemsError(err))?
        {
            self.elements
                .push(Arc::new(Mutex::new(System::from_sqlite_row(&row)?)));
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

        let player = Player::from_sqlite_row(first)?;

        let uuid = player.uuid;

        self.elements.push(Arc::new(Mutex::new(player)));

        Ok(uuid)
    }

    pub async fn leave(&mut self, uuid: Uuid) {
        let mut i = 0;
        for element in &self.elements {
            if element.lock().await.uuid == uuid {
                break;
            }
            i += 1;
        }

        assert!(i < self.elements.len());

        self.elements.remove(i);
    }

    pub async fn authenticate(instance: &mut Instance, nickname: &String) -> Result<Uuid> {
        let maybe_uuid = instance.load_player_by_nickname(nickname.clone()).await;

        match maybe_uuid {
            Err(Error::DbLoadPlayerByNicknameNotFound) => {
                let player_system = gen_system();
                let player_sys_uuid = player_system.uuid;
                instance.elements.push(Arc::new(Mutex::new(player_system)));

                let player = ElementContainer::new(
                    Element::Player(Player::new(
                        nickname.clone(),
                        player_sys_uuid,
                        player_sys_uuid,
                    )),
                    Uuid::new_v4(),
                    GalacticCoords::default(),
                );
                let uuid = player.uuid;
                instance.elements.push(Arc::new(Mutex::new(player)));
                instance.sync_to_db().await?;

                Ok(uuid)
            }
            Ok(uuid) => Ok(uuid),
            Err(err) => Err(err),
        }
    }

    pub async fn update(&mut self, delta: f32) -> bool {
        let mut has_changed = false;
        for element in &mut self.elements {
            if element.lock().await.update(delta) {
                element.lock().await.set_synced(true);
                has_changed = true;
            }
        }
        has_changed
    }
}

pub fn gen_system() -> ElementContainer {
    let mut rng = rand::thread_rng();
    let angle_1 = rng.gen_range(0..15000) as f64 / 10000.;
    let angle_2 = rng.gen_range(0..15000) as f64 / 10000.;
    let distance = rng.gen_range(0.0..10000000000.);

    ElementContainer::new(
        Element::System(System::new()),
        Uuid::new_v4(),
        GalacticCoords::new(angle_1, angle_2, distance),
    )
}
