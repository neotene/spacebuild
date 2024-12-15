use super::element::Element;
use super::elements::player::Player;
use super::galaxy::{gen_system, Galactic, Galaxy};
use super::repr::{GlobalCoords, LocalCoords};
use crate::error::Error;
use crate::Result;
use futures::TryStreamExt;
use is_printable::IsPrintable;
use log::trace;
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

        let suffix = "ON CONFLICT(uuid) DO UPDATE SET uuid=uuid";
        if let Some(sql_str) = insert_systems_sql_str.strip_suffix(", ") {
            let sql_str = format!("{} {}", sql_str, suffix);
            trace!("{}", sql_str);
            let _ = sqlx::query(&sql_str)
                .fetch_all(&self.pool)
                .await
                .map_err(|err| Error::DbSyncSystemsToDbError(err))?;
        }

        if let Some(sql_str) = insert_bodies_sql_str.strip_suffix(", ") {
            let sql_str = format!("{} {}", sql_str, suffix);
            trace!("{}", sql_str);
            let _ = sqlx::query(&sql_str)
                .fetch_all(&self.pool)
                .await
                .map_err(|err| Error::DbSyncBodiesToDbError(err))?;
        }

        if let Some(sql_str) = insert_players_sql_str.strip_suffix(", ") {
            let sql_str = format!("{} {}", sql_str, suffix);
            trace!("{}", sql_str);
            let _ = sqlx::query(&sql_str)
                .fetch_all(&self.pool)
                .await
                .map_err(|err| Error::DbSyncPlayersToDbError(err))?;
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
        if nickname.is_empty() || !nickname.is_printable() {
            return Err(Error::InvalidNickname);
        }

        for galactic in &self.galaxy.galactics {
            if let Element::Player(player) = &galactic.element {
                if player.nickname == nickname {
                    return Err(Error::PlayerAlreadyAuthenticated);
                }
            }
        }

        trace!("Trying to load player {} from DB...", nickname);
        let rows = sqlx::query("SELECT * FROM Player WHERE nickname=?")
            .bind(&nickname)
            .fetch_all(&self.pool)
            .await
            .map_err(|err| Error::DbLoadPlayerByNicknameQueryError(err))?;

        if rows.len() == 0 {
            return Err(Error::DbLoadPlayerByNicknameNotFound);
        }

        if rows.len() > 1 {
            unreachable!()
        }

        let first = rows.first().unwrap();

        let player = Galactic::from_sqlite_row(first)?;

        let uuid = player.uuid;

        self.galaxy.galactics.push(player);

        Ok(uuid)
    }

    pub async fn leave(&mut self, uuid: Uuid) -> Result<()> {
        let mut i = 0;
        for element in &self.galaxy.galactics {
            if element.uuid == uuid {
                break;
            }
            i += 1;
        }

        assert!(i < self.galaxy.galactics.len());

        self.sync_to_db().await?;

        self.galaxy.galactics.remove(i);

        Ok(())
    }

    pub async fn authenticate(&mut self, nickname: &String) -> Result<Uuid> {
        let maybe_uuid = self.load_player_by_nickname(nickname.clone()).await;

        match maybe_uuid {
            Err(Error::DbLoadPlayerByNicknameNotFound) => {
                let (player_system, bodies_in_system) = gen_system();
                let player_sys_uuid = player_system.uuid;
                self.galaxy.galactics.push(player_system);

                for body in bodies_in_system {
                    self.galaxy.galactics.push(body.clone());
                }

                let player = Galactic::new(
                    Element::Player(Player::new(
                        nickname.clone(),
                        player_sys_uuid,
                        player_sys_uuid,
                    )),
                    Uuid::new_v4(),
                    GlobalCoords::default(),
                    LocalCoords::default(),
                    0.,
                );
                let uuid = player.uuid;
                self.galaxy.galactics.push(player);

                Ok(uuid)
            }
            Ok(uuid) => Ok(uuid),
            Err(err) => Err(err),
        }
    }
}
