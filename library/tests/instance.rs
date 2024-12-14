#[cfg(test)]
use test_helpers_async::*;

#[before_all]
#[cfg(test)]
mod space_build_tests_instance {
    use std::{env, fs::File, str::FromStr};

    use common::trace;
    use futures_time::{future::FutureExt, time::Duration};
    use log::info;
    use nalgebra::Vector3;
    use spacebuild::game::{
        element::{self, Element},
        instance::Instance,
        repr::GlobalCoords,
    };
    use sqlx::SqlitePool;
    use uuid::Uuid;

    pub fn before_all() {
        trace::init(Some("(.*)".to_string()));
        info!("Timeout is {}s", TIMEOUT_DURATION);
    }

    const TIMEOUT_DURATION: u64 = 10;

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

        let instance = Instance::from_path(db_path.as_str())
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await??;

        let systems = instance.borrow_galaxy().borrow_systems();
        assert_eq!(0, systems.len());
        let players = instance.borrow_galaxy().borrow_players();
        assert_eq!(0, players.len());

        let pool = SqlitePool::connect(format!("sqlite:{}", db_path).as_str())
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await??;

        let result =
            sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='Player';")
                .fetch_all(&pool)
                .timeout(Duration::from_secs(TIMEOUT_DURATION))
                .await??;

        assert_eq!(1, result.len());

        let result =
            sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='System';")
                .fetch_all(&pool)
                .timeout(Duration::from_secs(TIMEOUT_DURATION))
                .await??;

        assert_eq!(1, result.len());

        let result =
            sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='Body';")
                .fetch_all(&pool)
                .timeout(Duration::from_secs(TIMEOUT_DURATION))
                .await??;

        assert_eq!(1, result.len());

        Ok(())
    }

    #[tokio::test]
    async fn case_02_load_systems() -> anyhow::Result<()> {
        let db_path = get_random_db_path();

        File::create(db_path.clone())?;

        let pool = SqlitePool::connect(format!("sqlite:{}", db_path).as_str())
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await??;

        sqlx::query(Instance::CREATE_TABLE_SYSTEM_SQL_STR)
            .execute(&pool)
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await??;

        let uuid_str = "e599a2ae-58a8-449f-8007-80de1ea791e9";

        sqlx::query("INSERT INTO System VALUES (?, 1.0, 2.0, 3.0, 4.0)")
            .bind(uuid_str)
            .execute(&pool)
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await??;

        let instance = Instance::from_path(db_path.as_str())
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await??;

        let uuid_1 = Uuid::from_str(uuid_str).unwrap();

        let systems = instance.borrow_galaxy().borrow_systems();
        assert_eq!(1, systems.len());

        let mut find = false;
        for system in systems {
            if system.get_uuid() == uuid_1 {
                find = true;
                break;
            }
        }
        assert!(find);

        let system = instance.borrow_galaxy().borrow_galactic(uuid_1).unwrap();

        assert_eq!(GlobalCoords::new(1., 2., 3.), system.get_coords());

        Ok(())
    }

    #[tokio::test]
    async fn case_03_save() -> anyhow::Result<()> {
        let db_path = get_random_db_path();

        let mut instance = Instance::from_path(db_path.as_str())
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await??;

        let sys_1 = element::System::default();

        let uuid_1 = instance
            .borrow_galaxy_mut()
            .add_galactic(Element::System(sys_1), GlobalCoords::new(1., 2., 3.));

        let sys_2 = element::System::default();

        let uuid_2 = instance
            .borrow_galaxy_mut()
            .add_galactic(Element::System(sys_2), GlobalCoords::new(4., 5., 6.));

        instance
            .sync_to_db()
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await??;

        let instance = Instance::from_path(db_path.as_str())
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await??;

        let systems = instance.borrow_galaxy().borrow_systems();
        assert_eq!(2, systems.len());
        let mut find = false;
        for system in &systems {
            if system.get_uuid() == uuid_1 {
                find = true;
                break;
            }
        }
        assert!(find);

        let system = instance.borrow_galaxy().borrow_galactic(uuid_1).unwrap();

        assert_eq!(GlobalCoords::new(1., 2., 3.), system.get_coords());

        let mut find = false;
        for system in &systems {
            if system.get_uuid() == uuid_2 {
                find = true;
                break;
            }
        }
        assert!(find);

        let system = instance.borrow_galaxy().borrow_galactic(uuid_2).unwrap();

        assert_eq!(GlobalCoords::new(4., 5., 6.), system.get_coords());

        Ok(())
    }

    #[tokio::test]
    async fn case_04_add_get() -> anyhow::Result<()> {
        let db_path = get_random_db_path();

        let mut instance = Instance::from_path(db_path.as_str())
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await??;

        let player = element::Player::new("player123".to_string(), Uuid::new_v4(), Uuid::new_v4());

        let uuid = instance
            .borrow_galaxy_mut()
            .add_galactic(Element::Player(player), GlobalCoords::new(1., 2., 3.));

        assert_eq!(1, instance.borrow_galaxy().borrow_players().len());
        assert_eq!(
            true,
            instance.borrow_galaxy().borrow_galactic(uuid).is_some()
        );

        let player_cmp = instance.borrow_galaxy().borrow_galactic(uuid).unwrap();

        assert_eq!(player_cmp.get_uuid(), uuid);
        assert_eq!(player_cmp.get_direction(), Vector3::new(0., 0., 0.));
        assert_eq!(player_cmp.get_coords(), GlobalCoords::new(1., 2., 3.));

        assert_eq!(0., player_cmp.get_speed());

        if let Element::Player(player_cmp) = player_cmp.get_element() {
            assert_eq!("player123", player_cmp.get_nickname());
        }

        Ok(())
    }

    #[tokio::test]
    async fn case_05_load_player_by_nickname() -> anyhow::Result<()> {
        let db_path = get_random_db_path();

        File::create(db_path.clone())?;

        let pool = SqlitePool::connect(format!("sqlite:{}", db_path).as_str())
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await??;

        sqlx::query(Instance::CREATE_TABLE_PLAYER_SQL_STR)
            .execute(&pool)
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await??;

        let uuid_str = "e599a2ae-58a8-449f-8007-80de1ea791e9";

        sqlx::query(
            "INSERT INTO Player VALUES (?, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 'player123', ?, ?)",
        )
        .bind(uuid_str)
        .bind(Uuid::new_v4().to_string())
        .bind(Uuid::new_v4().to_string())
        .execute(&pool)
        .timeout(Duration::from_secs(TIMEOUT_DURATION))
        .await??;

        let mut instance = Instance::from_path(db_path.as_str())
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await??;

        let uuid = instance
            .load_player_by_nickname("player123".to_string())
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await??;

        assert_eq!(uuid.to_string(), uuid_str);

        let player_cmp = instance.borrow_galaxy().borrow_galactic(uuid).unwrap();

        assert_eq!(player_cmp.get_uuid(), uuid);
        assert_eq!(player_cmp.get_direction(), Vector3::new(5., 6., 7.));
        assert_eq!(player_cmp.get_coords(), GlobalCoords::new(1., 2., 3.));
        assert_eq!(4., player_cmp.get_speed());

        if let Element::Player(player_cmp) = player_cmp.get_element() {
            assert_eq!("player123", player_cmp.get_nickname());
        }

        Ok(())
    }
}
