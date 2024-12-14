use super::element::{Body, Element};
use super::elements::body::BodyType;
use super::elements::player::Player;
use super::elements::system::System;
use super::galactic::{Galactic, Galaxy};
use super::repr::{GalacticCoords, LocalCoords};
use crate::error::Error;
use crate::Result;
use futures::TryStreamExt;
use rand::Rng;
use sqlx::{Pool, Sqlite, SqlitePool};
use std::fs::File;
use std::path::Path;
use uuid::Uuid;

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
        for element in &self.galaxy.galactics {
            let guard = element;
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

    pub fn borrow_galaxy(&self) -> &Galaxy {
        &self.galaxy
    }

    pub fn borrow_galaxy_mut(&mut self) -> &mut Galaxy {
        &mut self.galaxy
    }

    pub async fn load_systems(&mut self) -> Result<()> {
        let mut rows = sqlx::query("SELECT * FROM System").fetch(&self.pool);

        while let Some(row) = rows
            .try_next()
            .await
            .map_err(|err| Error::DbLoadSystemsError(err))?
        {
            self.galaxy.galactics.push(Galactic::from_sqlite_row(&row)?);
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

        let player = Galactic::from_sqlite_row(first)?;

        let uuid = player.uuid;

        for element in &self.galaxy.galactics {
            if element.uuid == uuid {
                return Err(Error::PlayerAlreadyAuthenticated);
            }
        }

        self.galaxy.galactics.push(player);

        Ok(uuid)
    }

    pub async fn leave(&mut self, uuid: Uuid) {
        let mut i = 0;
        for element in &self.galaxy.galactics {
            if element.uuid == uuid {
                break;
            }
            i += 1;
        }

        assert!(i < self.galaxy.galactics.len());

        self.galaxy.galactics.remove(i);
    }

    pub async fn authenticate(instance: &mut Instance, nickname: &String) -> Result<Uuid> {
        let maybe_uuid = instance.load_player_by_nickname(nickname.clone()).await;

        match maybe_uuid {
            Err(Error::DbLoadPlayerByNicknameNotFound) => {
                let (player_system, bodies_in_system) = gen_system();
                let player_sys_uuid = player_system.uuid;
                instance.galaxy.galactics.push(player_system);

                for body in bodies_in_system {
                    instance.galaxy.galactics.push(body);
                }

                let player = Galactic::new(
                    Element::Player(Player::new(
                        nickname.clone(),
                        player_sys_uuid,
                        player_sys_uuid,
                    )),
                    Uuid::new_v4(),
                    GalacticCoords::default(),
                    LocalCoords::default(),
                    0.,
                );
                let uuid = player.uuid;
                instance.galaxy.galactics.push(player);
                instance.sync_to_db().await?;

                Ok(uuid)
            }
            Ok(uuid) => Ok(uuid),
            Err(err) => Err(err),
        }
    }

    pub async fn update(&mut self, delta: f64) -> bool {
        let mut galaxy = self.galaxy.clone();
        for galactic in &mut self.galaxy.galactics {
            let galactic_nth = galaxy
                .galactics
                .iter()
                .position(|g| g.uuid == galactic.uuid)
                .unwrap();

            galaxy.galactics.remove(galactic_nth);
            galactic.update(delta, &galaxy).await;
        }
        false
    }
}

pub fn gen_system() -> (Galactic, Vec<Galactic>) {
    let mut rng = rand::thread_rng();
    let angle_1 = rng.gen_range(0..15000) as f64 / 10000.;
    let angle_2 = rng.gen_range(0..15000) as f64 / 10000.;
    let distance = rng.gen_range(0.0..10000000000.);

    let uuid = Uuid::new_v4();
    let coords = GalacticCoords::new(angle_1, angle_2, distance);

    let system = Galactic::new(
        Element::System(System::default()),
        uuid,
        coords.clone(),
        LocalCoords::default(),
        0.,
    );

    let mut bodies_in_system = Vec::<Galactic>::default();

    for _ in 1..100 {
        let x = rng.gen_range(0..1000) as f64;
        let y = rng.gen_range(0..1000) as f64;
        let z = rng.gen_range(0..10) as f64;

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
