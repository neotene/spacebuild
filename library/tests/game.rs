#[cfg(test)]
use test_helpers_async::*;

#[before_all]
#[cfg(test)]
mod spacebuild_tests_game {
    use std::{env, fs::File, sync::Arc};

    use anyhow::anyhow;
    use common::trace;
    use futures_time::{future::FutureExt, time::Duration};
    use log::info;
    use spacebuild::{
        client::Client,
        game::{
            element::{Element, Player},
            instance::Instance,
            repr::GalacticCoords,
        },
        network::tls::{ClientPki, ServerPki},
        protocol::GameInfo,
        server,
    };
    use tokio::{net::TcpListener, sync::Mutex};
    use uuid::Uuid;

    const SERVER_CERT: &[u8] = b"-----BEGIN CERTIFICATE-----
MIIDMTCCAhmgAwIBAgIUPW2I5vQZWOxWMHqP1Pu73GfKvhUwDQYJKoZIhvcNAQEL
BQAwHTELMAkGA1UEBhMCRkkxDjAMBgNVBAMMBXZhaGlkMB4XDTI0MTIwMTIwMzAw
NFoXDTI1MTIwMTIwMzAwNFowHTELMAkGA1UEBhMCRkkxDjAMBgNVBAMMBXZhaGlk
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAtcLRSlxRbbOT4m1vKeWm
HRxSpr6YdHT4TlJYcQnvNg7NQBoSQFLTY/c9vDwnwpC3nDc+I3VauZYb44Iocnht
BK7AQPyscjM6dwVu0mxFIgc0i2t5+yrNs8n5jWzHsMu7ZgMc9RmRBzgadw/9VHcH
RyFJt1wYIJI48PjNW/IfzeGYCNEjTdWYifBdZKt4gOrpcEvHzlsjebcVdXTrS8sI
82zLKCGfy07JqDxHhMb4uIb/J/SKNkng2Dpr9Ythxfn5dD4BKuaKrEnxjLxBKX3J
SUa5+bs3lP/LH5nz/cogBV6t6BIoJ7p//jgjSalCkXvGnKG/+asid1JJ0z5ZuM/R
KwIDAQABo2kwZzAfBgNVHSMEGDAWgBQ6XXVXE9iMux7aLuk0hcRz42f+JTAJBgNV
HRMEAjAAMBoGA1UdEQQTMBGCCWxvY2FsaG9zdIcEfwAAATAdBgNVHQ4EFgQU6Yab
dvv0NBb/mYRdbOzN3T+gUcYwDQYJKoZIhvcNAQELBQADggEBAFLoifH57rdSzLV/
ZuOGEKvn/KgAcM+p+Sj7vujwe+vntMMBSjChm98YsOPR26j0aweKfHlnrbPuerk1
dvU34pe0v0TDzLIpJuIkfZe5MMx3WjvhwTPOWlAqxaMMxAD+95I6KChP4lV9xqLv
iPgSDSODElS/qKb3kU4sA4m2CxmI6yCWW2tYsjoTkqrBmhjKql6UnBBrkb5K6tXm
jcg0sq+u24j0Hzq9slk3Uxk3viqdN1X6p1sPCeAdO7Q2y6NBB8rTYu6klUQQRWL8
NH4has89I4jp2ufcy1zY4ckN3uSZffG8S+v3jv/c9dmZoV7OO1CYnwvzgo01k9GD
Vqi4i7M=
-----END CERTIFICATE-----
";

    const SERVER_KEY: &[u8] = b"-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQC1wtFKXFFts5Pi
bW8p5aYdHFKmvph0dPhOUlhxCe82Ds1AGhJAUtNj9z28PCfCkLecNz4jdVq5lhvj
gihyeG0ErsBA/KxyMzp3BW7SbEUiBzSLa3n7Ks2zyfmNbMewy7tmAxz1GZEHOBp3
D/1UdwdHIUm3XBggkjjw+M1b8h/N4ZgI0SNN1ZiJ8F1kq3iA6ulwS8fOWyN5txV1
dOtLywjzbMsoIZ/LTsmoPEeExvi4hv8n9Io2SeDYOmv1i2HF+fl0PgEq5oqsSfGM
vEEpfclJRrn5uzeU/8sfmfP9yiAFXq3oEignun/+OCNJqUKRe8acob/5qyJ3UknT
Plm4z9ErAgMBAAECggEAHOKT/hxDuIpUUySPCPp89p1cqTEa6073cwL1GSm6AT5C
8g/ynJRNEdLl1bc9nlb/Ru0ki+AHhfzL+9DgeqiWsqrO1MUS5qcrgGS1ou0f43N/
rzRqUzcPL6ZGaWpDJd6KroCKJo1kleAdnJRG7xhnaK9qlqAlGXADapAvmpAU69PM
MwpW9S96QvVHfPP7LXO/nvNzqLnrNysprHkSH6iV4ao37LEqzgUF0tABTk0Q67UJ
O4XSToMAJ8GOBjYSKVK3PJm3saqTobff9Oz2HgUWUyr92kSESPhfNEVlMskmgvE3
CcajxOxudxg94AAU7Es1UE5bMtY2e/Cs1088yzC3SQKBgQDvtYHI+4Kcur2ply0p
QIBSSspJZ7fGT9/waK0EFlAyQ/qAaFH0Ilb6U2/L52TSR0EbSImQN7VxkUrosHym
HahB6yHXkI2G8nDcmSdNjyiiC00+LWyKCtixE+bRCAuReZmypSk1Fz8GwYb3gaBR
YcsWGsMeomFpL6q6yIgo43r8xQKBgQDCHR9fciT7zHTWAyPNlPLVzuJlvi164OC8
GkHHxx+CybIDZVrUdUfYk80kxC+bvlUIaMs2D0MVUg2Hv8IbtMjEs+FV4vM/Df9J
e9SWhOTWz25Jc7ZRYKVKc848l6TQd5JMU4JjeqmmVAza27l6Iu4TQb+r9GrZgBxX
6NBj8vZVLwKBgFsW1iLRsGhubfQsBnVOlXSwBv6t8x/g6nAo1tZexErVmjOBcOMc
yYCGhE0vuRhPC2aaweuTv9dQJu8VYcieLHogJ9QKkj1dk5XAfTbz17T8JnYiPMSY
Ko/fyC5WqE63rrg8GtSZ6NFgaTFUiN9kEhBsSwkxG2MlQfOIkHU5PFshAoGBAL6c
4GjWapDERdq9/JNs90STQmgMZxap6qVr1zp5Q20n6GFDTv0gKav3/1NiPyndrhxy
41GzjPlLuLObzt1sGlZmGRlAogJCGXSsX6Zq21hBGxiPwvGISOeiblu7wYFgWU4Q
FxLeqecF1BW5/Bl+YXCReMk/Wwk3rx14JeJv/ArLAoGAPwBXLX1HwQeHoFn4ImZV
r0fUKkD4LzaAJ4gbEqzAQ8AD8vmqq+CBpu1YCLO6SFqHsFj1RUfk1ScVVD9tlL7E
CI5ivNoxDpThvZhP6v42T7JQKK49YaGySE/k3y0wztfsk8qn6dAI6pwFMgtfsFFo
RZb6vjD6zPWZElSkrwGczDM=
-----END PRIVATE KEY-----
";

    const CA_CERT: &[u8] = b"-----BEGIN CERTIFICATE-----
MIIDGzCCAgOgAwIBAgIUVlpyalwiQIyyrcHPGXGm+1fEPMIwDQYJKoZIhvcNAQEL
BQAwHTELMAkGA1UEBhMCRkkxDjAMBgNVBAMMBXZhaGlkMB4XDTI0MTIwMTIwMjEy
NVoXDTI5MTEzMDIwMjEyNVowHTELMAkGA1UEBhMCRkkxDjAMBgNVBAMMBXZhaGlk
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAk/W74DzJBDOw5OW+EXSN
gMAfmgZnRc6sP698IcrsBFs78VqB0donQqltnD43Ohxe+iHDGdHI1H4I3dY3OgCY
HSIibJEkCfO4z1A3NtsNI8y2+AO3QKhMm9XK4TwMW9aFCnaocB+SbIbfmSiW5tfU
KXfVp8ya0ieAO5zTEkhXX6ZGqr1gFtyM7wx3pjUuzffMnFQPrIZoY9JxBe3qnPED
mkjC5qTxKytAfb6PpYYSl+jhnykfsMyR9IrypwUIG+IXImPd8y/6+m6JN06fwQWV
p49hu3XvvtGOEU23tEbgDQR5t0AjKMlHmT2Y0WG6GsAnDALnNBkGq7ZNrk17Mw91
VQIDAQABo1MwUTAdBgNVHQ4EFgQUOl11VxPYjLse2i7pNIXEc+Nn/iUwHwYDVR0j
BBgwFoAUOl11VxPYjLse2i7pNIXEc+Nn/iUwDwYDVR0TAQH/BAUwAwEB/zANBgkq
hkiG9w0BAQsFAAOCAQEAH0QgIq509cxFwSxqZRpbLBuHbdUq+xFB42N0ttDNJZzi
T01OWsPYtim8/WXlYC5PHv1FZthY9/7Ci2tEicm6X01CNnvNgeZx8bBGpOq0rqkY
+9xRPSQXVoIbApg3KHDeUq6Fe9leASFohEbXk7gbi9c1yuT4Z+O19KmY8/rtvR1N
U9c0sNvcDC5Q4bVai6KAhLxzLCBaYSqY4ku881K3pBSNVEy5gBVj466DOFNLPNg6
Oha9NBAsvMsXonrrYDYtwk92p3L9O55b/YKG0MYW4qCB27SZnYZwDea9+h/MLvFV
lBjhUjWT859gkyO6pYSTfndSpnWAdtQK9zsTYociBQ==
-----END CERTIFICATE-----
";

    pub fn before_all() {
        trace::init(Some(".*spacebuild(.*)".to_string()));
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

    async fn bootstrap(
        db_path: String,
        tls: bool,
    ) -> anyhow::Result<(
        Arc<Mutex<Instance>>,
        crossbeam::channel::Sender<()>,
        tokio::task::JoinHandle<spacebuild::Result<()>>,
        u16,
    )> {
        let listener = TcpListener::bind("localhost:0")
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await??;
        let addr = listener.local_addr()?;
        let port = addr.port();

        let instance = Arc::new(Mutex::new(
            Instance::from_path(db_path.as_str())
                .timeout(Duration::from_secs(TIMEOUT_DURATION))
                .await??,
        ));

        let instance_cln = Arc::clone(&instance);

        let pki = if tls {
            Some(ServerPki::Slices {
                key: SERVER_KEY,
                cert: SERVER_CERT,
            })
        } else {
            None
        };
        let (send_stop, recv_stop) = crossbeam::channel::bounded(1);
        let game_thread: tokio::task::JoinHandle<spacebuild::Result<()>> =
            tokio::spawn(async move {
                server::run(
                    server::InstanceConfig::UserInstance(instance_cln),
                    server::ServerConfig {
                        tcp: server::TcpConfig::TcpListener(listener),
                        pki,
                    },
                    false,
                    recv_stop,
                )
                .await?;
                Ok(())
            });

        Ok((instance, send_stop, game_thread, port))
    }

    #[tokio::test]
    async fn case_01_connection() -> anyhow::Result<()> {
        let db_path = get_random_db_path();

        let (_instance, send_stop, game_thread, port) = bootstrap(db_path, false)
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await??;

        Client::connect(format!("localhost:{}", port).as_str(), None)
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await??;

        send_stop.send(())?;

        game_thread
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await???;

        Ok(())
    }

    #[tokio::test]
    async fn case_02_double_connection() -> anyhow::Result<()> {
        let db_path = get_random_db_path();

        let (_instance, send_stop, game_thread, port) = bootstrap(db_path, false)
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await??;

        let _ = Client::connect(format!("localhost:{}", port).as_str(), None)
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await??;

        Client::connect(format!("localhost:{}", port).as_str(), None)
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await??;

        send_stop.send(())?;

        game_thread
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await???;

        Ok(())
    }

    #[tokio::test]
    async fn case_03_successful_first_authentication() -> anyhow::Result<()> {
        let db_path = get_random_db_path();

        let (_instance, send_stop, game_thread, port) = bootstrap(db_path, false)
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await??;

        let mut player = Client::connect(format!("localhost:{}", port).as_str(), None)
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await??;

        player
            .login("test")
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await??;

        send_stop.send(())?;

        game_thread
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await???;

        Ok(())
    }

    #[tokio::test]
    async fn case_04_successful_first_authentication_tls() -> anyhow::Result<()> {
        let db_path = get_random_db_path();

        let (_instance, send_stop, game_thread, port) = bootstrap(db_path, true)
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await??;

        let mut player = Client::connect(
            format!("localhost:{}", port).as_str(),
            Some(ClientPki::Slice { cert: CA_CERT }),
        )
        .timeout(Duration::from_secs(TIMEOUT_DURATION))
        .await??;

        player
            .login("test")
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await??;

        send_stop.send(())?;

        game_thread
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await???;

        Ok(())
    }

    #[tokio::test]
    async fn case_05_known_player_authentication() -> anyhow::Result<()> {
        let db_path = get_random_db_path();

        {
            File::create(db_path.clone())?;

            let mut instance = Instance::from_path(db_path.as_str())
                .timeout(Duration::from_secs(TIMEOUT_DURATION))
                .await??;

            instance.borrow_galaxy_mut().add_galactic(
                Element::Player(Player::new(
                    "test963".to_string(),
                    Uuid::new_v4(),
                    Uuid::new_v4(),
                )),
                GalacticCoords::default(),
            );

            instance
                .sync_to_db()
                .timeout(Duration::from_secs(TIMEOUT_DURATION))
                .await??;
        }

        let (_instance, send_stop, game_thread, port) = bootstrap(db_path, false)
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await??;

        let mut player = Client::connect(format!("localhost:{}", port).as_str(), None)
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await??;

        player
            .login("test")
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await??;

        send_stop.send(())?;

        game_thread
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await???;

        Ok(())
    }

    #[tokio::test]
    async fn case_06_double_authentication() -> anyhow::Result<()> {
        let db_path = get_random_db_path();

        let (_instance, send_stop, game_thread, port) = bootstrap(db_path, false)
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await??;

        let mut player = Client::connect(format!("localhost:{}", port).as_str(), None)
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await??;

        assert!(player.login("test").await.is_ok());

        assert!(player.login("test").await.is_err());

        send_stop.send(())?;

        game_thread
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await???;

        Ok(())
    }

    #[tokio::test]
    async fn case_07_auth_reauth() -> anyhow::Result<()> {
        let db_path = get_random_db_path();

        let (_instance, send_stop, game_thread, port) = bootstrap(db_path, false)
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await??;

        let uuid1;
        {
            let mut player = Client::connect(format!("localhost:{}", port).as_str(), None)
                .timeout(Duration::from_secs(TIMEOUT_DURATION))
                .await??;
            uuid1 = player
                .login("test")
                .timeout(Duration::from_secs(TIMEOUT_DURATION))
                .await??;
            player
                .terminate()
                .timeout(Duration::from_secs(TIMEOUT_DURATION))
                .await??;
        }

        let uuid2;
        {
            let mut player = Client::connect(format!("localhost:{}", port).as_str(), None)
                .timeout(Duration::from_secs(TIMEOUT_DURATION))
                .await??;
            uuid2 = player
                .login("test")
                .timeout(Duration::from_secs(TIMEOUT_DURATION))
                .await??;
            player
                .terminate()
                .timeout(Duration::from_secs(TIMEOUT_DURATION))
                .await??;
        }

        assert_eq!(uuid1, uuid2);
        send_stop.send(())?;

        game_thread
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await???;

        Ok(())
    }

    #[tokio::test]
    async fn case_08_wait_first_gameinfo() -> anyhow::Result<()> {
        let db_path = get_random_db_path();

        let (_instance, _send_stop, game_thread, port) = bootstrap(db_path, false)
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await??;

        let mut player = Client::connect(format!("localhost:{}", port).as_str(), None)
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await??;
        player
            .login("test")
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await??;

        let game_info = player
            .next_game_info()
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await
            .map_err(|_| anyhow!("Waited first game info for too long"))?;

        if let GameInfo::Player(_player_info) = game_info.unwrap() {
        } else {
            assert!(false)
        }

        player
            .terminate()
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await??;

        game_thread
            .timeout(Duration::from_secs(TIMEOUT_DURATION))
            .await???;

        Ok(())
    }
}
