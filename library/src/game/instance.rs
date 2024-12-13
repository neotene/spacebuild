use crate::error::Error;
use crate::protocol::gen_system;
use crate::Result;
use futures::TryStreamExt;
use regex::Regex;
use sqlx::{Pool, Sqlite, SqlitePool};
use std::fs::File;
use std::path::Path;
use uuid::Uuid;
extern crate downcast;

use super::element::Element;
use super::elements::player::Player;
use super::elements::system::System;
use super::repr::GalacticCoords;

type RefType<T> = Box<T>;
type ElementsCollection = Vec<RefType<dyn Element>>;
pub struct Instance {
    pub(crate) pool: Pool<Sqlite>,
    pub(crate) elements: ElementsCollection,
}

unsafe impl Send for Instance {}
unsafe impl Sync for Instance {}

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
        self.save_players().await?;
        self.save_systems().await?;
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

    fn get_mut_element_of<T>(&mut self, uuid: Uuid) -> Option<&mut T>
    where
        T: Element,
    {
        for element in &mut self.elements {
            match element.downcast_mut::<T>() {
                Ok(true_element) => {
                    if true_element.get_uuid() == uuid {
                        return Some(true_element);
                    }
                }
                Err(_err) => {}
            }
        }
        None
    }

    fn get_element_of<T>(&self, uuid: Uuid) -> Option<&T>
    where
        T: Element + Clone + 'static,
    {
        for element in &self.elements {
            match element.downcast_ref::<T>() {
                Ok(true_element) => {
                    if true_element.get_uuid() == uuid {
                        return Some(true_element);
                    }
                }
                Err(_err) => {}
            }
        }
        None
    }

    fn get_elements_of<T>(&self) -> Vec<&T>
    where
        T: Element,
    {
        let mut reduced = Vec::new();
        for element in &self.elements {
            match element.downcast_ref::<T>() {
                Ok(true_element) => reduced.push(true_element),
                Err(_err) => {}
            }
        }
        reduced
    }

    fn get_mut_elements_of<T>(&mut self) -> Vec<&mut T>
    where
        T: Element,
    {
        let mut reduced = Vec::new();
        for element in &mut self.elements {
            match element.downcast_mut::<T>() {
                Ok(true_element) => reduced.push(true_element),
                Err(_err) => {}
            }
        }
        reduced
    }

    pub fn get_elements(&self) -> &ElementsCollection {
        &self.elements
    }

    async fn save_elements_of<T>(&mut self) -> Result<()>
    where
        T: Element,
    {
        let mut query = Self::get_insert_query::<T>();

        self.get_elements_of::<T>().iter().for_each(|element| {
            if !element.is_synced() {
                query.push_str(element.get_sql_insert_line().as_str());
            }
        });

        if query.pop().unwrap() == ',' {
            sqlx::query(query.as_str())
                .execute(&self.pool)
                .await
                .map_err(|err| Error::DbSyncSystemsToDbError(err))?;

            self.get_mut_elements_of::<T>()
                .iter_mut()
                .for_each(|element| {
                    element.set_synced(true);
                });
        }

        Ok(())
    }

    pub fn get_system(&self, uuid: Uuid) -> Option<&System> {
        self.get_element_of(uuid)
    }

    pub fn get_mut_player(&mut self, uuid: Uuid) -> Option<&mut Player> {
        self.get_mut_element_of(uuid)
    }

    pub fn get_player(&self, uuid: Uuid) -> Option<&Player> {
        self.get_element_of(uuid)
    }

    pub fn get_systems(&self) -> Vec<&System> {
        self.get_elements_of()
    }

    pub fn get_players(&self) -> Vec<&Player> {
        self.get_elements_of()
    }

    pub fn add_system(&mut self, system: System) {
        self.elements.push(RefType::new(system));
    }

    pub fn add_player(&mut self, player: Player) {
        self.elements.push(RefType::new(player));
    }

    pub async fn save_systems(&mut self) -> Result<()> {
        self.save_elements_of::<System>().await
    }

    pub async fn save_players(&mut self) -> Result<()> {
        self.save_elements_of::<Player>().await
    }

    pub async fn load_systems(&mut self) -> Result<()> {
        let mut rows = sqlx::query("SELECT * FROM System").fetch(&self.pool);

        while let Some(row) = rows
            .try_next()
            .await
            .map_err(|err| Error::DbLoadSystemsError(err))?
        {
            self.elements
                .push(RefType::new(System::from_sqlite_row(&row)?));
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

        let uuid = player.get_uuid();

        let players = self.get_elements_of::<Player>();
        if players.iter().filter(|p| p.nickname() == nickname).count() > 0 {
            return Err(Error::PlayerAlreadyAuthenticated);
        }

        self.elements.push(RefType::new(player));

        Ok(uuid)
    }

    pub async fn leave(&mut self, uuid: Uuid) {
        let player_element_nth = self
            .elements
            .iter()
            .position(|e| e.get_uuid() == uuid)
            .unwrap();

        self.elements.remove(player_element_nth);
    }

    pub async fn authenticate(instance: &mut Instance, nickname: &String) -> Result<Uuid> {
        let maybe_uuid = instance.load_player_by_nickname(nickname.clone()).await;

        match maybe_uuid {
            Err(Error::DbLoadPlayerByNicknameNotFound) => {
                let player_system = gen_system();
                let player_sys_uuid = player_system.get_uuid();
                instance.add_system(player_system);
                instance.save_systems().await?;

                let player =
                    Player::new(GalacticCoords::default(), nickname.clone(), player_sys_uuid);
                let uuid = player.get_uuid();
                instance.add_player(player);
                instance.save_players().await?;

                Ok(uuid)
            }
            Ok(uuid) => Ok(uuid),
            Err(err) => Err(err),
        }
    }

    pub fn update(&mut self, delta: f32) -> bool {
        let mut has_changed = false;
        for element in &mut self.elements {
            if element.update(delta) {
                element.set_synced(true);
                has_changed = true;
            }
        }
        has_changed
    }
}
