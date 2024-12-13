#[cfg(test)]
use test_helpers_async::*;

#[before_all]
#[cfg(test)]
mod space_build_tests_instance {
    use std::{env, fs::File, str::FromStr};

    use common::trace;
    use nalgebra::Vector3;
    use spacebuild::game::{
        element::{self, Element},
        instance::Instance,
        repr::GalacticCoords,
    };
    use sqlx::SqlitePool;
    use uuid::Uuid;

    pub fn before_all() {
        trace::init(Some("(.*)".to_string()));
    }

    pub fn get_random_db_path() -> String {
        format!(
            "{}space_build_tests_{}.db",
            env::temp_dir().to_str().unwrap(),
            Uuid::new_v4().to_string()
        )
    }

    #[tokio::test]
    async fn case_01_db_init() -> anyhow::Result<()> {
        let db_path = get_random_db_path();

        let instance = Instance::from_path(db_path.as_str()).await?;

        let systems = instance.get_systems().await;
        assert_eq!(0, systems.len());
        let players = instance.get_players().await;
        assert_eq!(0, players.len());

        let pool = SqlitePool::connect(format!("sqlite:{}", db_path).as_str()).await?;

        let result =
            sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='Player';")
                .fetch_all(&pool)
                .await?;

        assert_eq!(1, result.len());

        let result =
            sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='System';")
                .fetch_all(&pool)
                .await?;

        assert_eq!(1, result.len());

        let result =
            sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='Body';")
                .fetch_all(&pool)
                .await?;

        assert_eq!(1, result.len());

        Ok(())
    }

    #[tokio::test]
    async fn case_02_load_systems() -> anyhow::Result<()> {
        let db_path = get_random_db_path();

        File::create(db_path.clone())?;

        let pool = SqlitePool::connect(format!("sqlite:{}", db_path).as_str()).await?;

        sqlx::query(Instance::CREATE_TABLE_SYSTEM_SQL_STR)
            .execute(&pool)
            .await?;

        let uuid_str = "e599a2ae-58a8-449f-8007-80de1ea791e9";

        sqlx::query("INSERT INTO System VALUES (?, 1.0, 2.0, 3.0, 4.0)")
            .bind(uuid_str)
            .execute(&pool)
            .await?;

        let instance = Instance::from_path(db_path.as_str()).await?;

        let uuid_1 = Uuid::from_str(uuid_str).unwrap();

        let systems = instance.get_systems().await;
        assert_eq!(1, systems.len());

        let mut find = false;
        for system in systems {
            if system.lock().await.get_uuid() == uuid_1 {
                find = true;
                break;
            }
        }
        assert!(find);

        let system = instance.get_element(uuid_1).await.unwrap();

        assert_eq!(
            GalacticCoords::new(1., 2., 3.),
            system.lock().await.get_coords()
        );

        Ok(())
    }

    #[tokio::test]
    async fn case_03_save() -> anyhow::Result<()> {
        let db_path = get_random_db_path();

        let mut instance = Instance::from_path(db_path.as_str()).await?;

        let sys_1 = element::System::default();

        let uuid_1 = instance.add_element(Element::System(sys_1), GalacticCoords::new(1., 2., 3.));

        let sys_2 = element::System::default();

        let uuid_2 = instance.add_element(Element::System(sys_2), GalacticCoords::new(4., 5., 6.));

        instance.sync_to_db().await?;

        let instance = Instance::from_path(db_path.as_str()).await?;

        let systems = instance.get_systems().await;
        assert_eq!(2, systems.len());
        let mut find = false;
        for system in &systems {
            if system.lock().await.get_uuid() == uuid_1 {
                find = true;
                break;
            }
        }
        assert!(find);

        let system = instance.get_element(uuid_1).await.unwrap();

        assert_eq!(
            GalacticCoords::new(1., 2., 3.),
            system.lock().await.get_coords()
        );

        let mut find = false;
        for system in &systems {
            if system.lock().await.get_uuid() == uuid_2 {
                find = true;
                break;
            }
        }
        assert!(find);

        let system = instance.get_element(uuid_2).await.unwrap();

        assert_eq!(
            GalacticCoords::new(4., 5., 6.),
            system.lock().await.get_coords()
        );

        Ok(())
    }

    #[tokio::test]
    async fn case_04_add_get() -> anyhow::Result<()> {
        let db_path = get_random_db_path();

        let mut instance = Instance::from_path(db_path.as_str()).await?;

        let player = element::Player::new("player123".to_string(), Uuid::new_v4(), Uuid::new_v4());

        let uuid = instance.add_element(Element::Player(player), GalacticCoords::new(1., 2., 3.));

        assert_eq!(1, instance.get_elements().len());
        assert_eq!(true, instance.get_element(uuid).await.is_some());

        let player_cmp = instance.get_element(uuid).await.unwrap();

        assert_eq!(player_cmp.lock().await.get_uuid(), uuid);
        assert_eq!(
            player_cmp.lock().await.get_direction(),
            Vector3::new(0., 0., 0.)
        );
        assert_eq!(
            player_cmp.lock().await.get_coords(),
            GalacticCoords::new(1., 2., 3.)
        );

        assert_eq!(0., player_cmp.lock().await.get_speed());

        if let Element::Player(player_cmp) = player_cmp.lock().await.get_element() {
            assert_eq!("player123", player_cmp.get_nickname());
        }

        Ok(())
    }

    #[tokio::test]
    async fn case_05_load_player_by_nickname() -> anyhow::Result<()> {
        let db_path = get_random_db_path();

        File::create(db_path.clone())?;

        let pool = SqlitePool::connect(format!("sqlite:{}", db_path).as_str()).await?;

        sqlx::query(Instance::CREATE_TABLE_PLAYER_SQL_STR)
            .execute(&pool)
            .await?;

        let uuid_str = "e599a2ae-58a8-449f-8007-80de1ea791e9";

        sqlx::query(
            "INSERT INTO Player VALUES (?, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 'player123', ?, ?)",
        )
        .bind(uuid_str)
        .bind(Uuid::new_v4().to_string())
        .bind(Uuid::new_v4().to_string())
        .execute(&pool)
        .await?;

        let mut instance = Instance::from_path(db_path.as_str()).await?;

        let uuid = instance
            .load_player_by_nickname("player123".to_string())
            .await?;

        assert_eq!(uuid.to_string(), uuid_str);

        let player_cmp = instance.get_element(uuid).await.unwrap();

        assert_eq!(player_cmp.lock().await.get_uuid(), uuid);
        assert_eq!(
            player_cmp.lock().await.get_direction(),
            Vector3::new(5., 6., 7.)
        );
        assert_eq!(
            player_cmp.lock().await.get_coords(),
            GalacticCoords::new(1., 2., 3.)
        );
        assert_eq!(4., player_cmp.lock().await.get_speed());

        if let Element::Player(player_cmp) = player_cmp.lock().await.get_element() {
            assert_eq!("player123", player_cmp.get_nickname());
        }

        Ok(())
    }
}
