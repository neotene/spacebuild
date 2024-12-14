use crate::game::element::Element;
use crate::game::instance::Instance;
use crate::protocol::AuthInfo;
use crate::protocol::PlayerAction;
use futures::SinkExt;
use futures::StreamExt;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response, StatusCode};
use hyper_tungstenite::tungstenite::Message;
use hyper_tungstenite::HyperWebsocket;
use log::debug;
use log::error;
use log::info;
use log::trace;
use std::ops::DerefMut;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

pub async fn serve_http(
    mut request: Request<hyper::body::Incoming>,
    instance: Arc<Mutex<Instance>>,
) -> hyper::Result<Response<Full<Bytes>>> {
    let response_body = Full::<Bytes>::new("".into());
    let mut response = Response::<Full<Bytes>>::new(response_body);
    *response.status_mut() = StatusCode::BAD_REQUEST;

    if hyper_tungstenite::is_upgrade_request(&request) {
        info!("Upgrade request");
        let res = hyper_tungstenite::upgrade(&mut request, None);
        if res.is_err() {
            let err_str: String = res.err().unwrap().to_string();

            *response.body_mut() =
                Full::<Bytes>::new(format!("Can't upgrade to websocket: {}", err_str).into());

            info!("WS upgrade error");
            return Ok(response);
        }

        let (ws_resp, websocket) = res.unwrap();

        tokio::spawn(async move {
            let instance_cln = Arc::clone(&instance);
            serve_websocket(websocket, instance_cln).await;
        });

        return Ok(ws_resp);
    } else {
        *response.body_mut() = Full::<Bytes>::new(format!("Websocket only").into());
        info!("HTTP non WS request");
        return Ok(response);
    }
}

async fn serve_websocket(websocket: HyperWebsocket, instance: Arc<Mutex<Instance>>) {
    let maybe_websocket = websocket.await;
    if maybe_websocket.is_err() {
        error!("Websocket error: {}", maybe_websocket.err().unwrap());
        return;
    }
    let mut websocket = maybe_websocket.unwrap();

    let mut uuid = Uuid::max();

    let mut tick_delay = tokio::time::interval(std::time::Duration::from_millis(250));

    loop {
        tokio::select! {
            _ = tick_delay.tick() => {
                trace!("Service tick for {}", uuid.to_string());
                if uuid.is_max() {
                    continue;
                }
                trace!("0");
                let mut guard = instance.lock().await;
                let maybe_player = guard.borrow_galaxy_mut().borrow_galactic_mut(uuid).await;

                trace!("1");
                if let Some(player) = maybe_player {
                    trace!("2");
                    if let Element::Player(player) = &mut player.element {
                        trace!("3");
                        for game_info in &player.game_infos {
                            trace!("4");
                            let game_info_str = serde_json::to_string(&game_info).unwrap();
                            if websocket.send(Message::text(game_info_str)).await.is_err() {
                                error!("Could not send to client");
                            }
                        }
                        player.game_infos.clear();
                    }
                } else {
                    unreachable!()
                }
            },
            Some(message) = websocket.next() => {
                if message.is_err() {
                    trace!("Websocket read error: {}", message.err().unwrap());
                    break;
                }
                match message.unwrap() {
                    Message::Text(msg) => {
                        let maybe_action: serde_json::Result<PlayerAction> =
                            serde_json::from_str(msg.as_str());

                        let mut login_info = AuthInfo {
                            success: false,
                            message: "".to_string(),
                        };

                        if maybe_action.is_err() {
                            login_info.message = "Invalid JSON".to_string();
                        } else {
                            let maybe_login = maybe_action.unwrap();

                            if let PlayerAction::Login(login) = maybe_login {
                                info!("Login request");
                                let maybe_uuid = Instance::authenticate(
                                    instance.lock().await.deref_mut(),
                                    &login.nickname,
                                )
                                .await;
                                if maybe_uuid.is_err() {
                                    login_info.message = format!("{}", maybe_uuid.err().unwrap());
                                    info!("Login error: {}", login_info.message);
                                } else {
                                    uuid = maybe_uuid.unwrap();

                                    info!("Login success for {}", uuid);

                                    login_info.success = true;
                                    login_info.message = uuid.to_string();
                                }
                            } else {
                                let mut instance = instance.lock().await;
                                let maybe_element = instance.borrow_galaxy_mut().borrow_galactic_mut(uuid).await;
                                if let Some(maybe_player) = maybe_element {
                                    if let Element::Player(player) =
                                        &mut maybe_player.element
                                    {
                                        player.actions.push(maybe_login);
                                    }
                                } else {
                                    error!("Can't find player {}", uuid);
                                }
                            }
                        }
                        let maybe_login_info_str = serde_json::to_string(&login_info);
                        assert!(maybe_login_info_str.is_ok());
                        let result = websocket
                            .send(Message::text(maybe_login_info_str.unwrap()))
                            .await;
                        if result.is_err() {
                            debug!("Message send error: {}", result.err().unwrap());
                        }
                    }
                    Message::Binary(_msg) => {}
                    Message::Ping(_msg) => {}
                    Message::Pong(_msg) => {}
                    Message::Close(_msg) => {
                        info!("WS close request received");
                        if !uuid.is_max() {
                            instance.lock().await.leave(uuid).await;
                        }
                        break;
                    }
                    Message::Frame(_msg) => {}
                }
            }
        }
    }
}
