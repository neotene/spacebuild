use crate::game::element::Element;
use crate::game::instance::Instance;
use crate::protocol::AuthInfo;
use crate::protocol::PlayerAction;
use futures::SinkExt;
use futures::StreamExt;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response, StatusCode};
use hyper_tungstenite::tungstenite;
use hyper_tungstenite::tungstenite::Message;
use hyper_tungstenite::HyperWebsocket;
use log::debug;
use log::error;
use log::info;
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
            if let Err(err) = serve_websocket(websocket, instance_cln).await {
                log::info!("WS server error: {}", err);
            }
            ()
        });

        return Ok(ws_resp);
    } else {
        *response.body_mut() = Full::<Bytes>::new(format!("Websocket only").into());
        info!("HTTP non WS request");
        return Ok(response);
    }
}

async fn serve_websocket(
    websocket: HyperWebsocket,
    instance: Arc<Mutex<Instance>>,
) -> Result<(), tungstenite::error::Error> {
    let mut websocket = websocket.await?;
    let mut uuid = Uuid::max();

    while let Some(message) = websocket.next().await {
        match message? {
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
                        let instance = instance.lock().await;
                        let maybe_element = instance.get_element(uuid).await;
                        if let Some(maybe_player) = maybe_element {
                            if let Element::Player(player) =
                                &mut maybe_player.lock().await.deref_mut().element
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
                websocket
                    .send(Message::text(maybe_login_info_str.unwrap()))
                    .await?;
                debug!("Message sent");
            }
            Message::Binary(_msg) => {
                websocket
                    .send(Message::binary(b"Thank you, come again.".to_vec()))
                    .await?;
            }
            Message::Ping(_msg) => {}
            Message::Pong(_msg) => {}
            Message::Close(_msg) => {
                info!("WS close request received");
                if !uuid.is_max() {
                    instance.lock().await.leave(uuid).await;
                }
                break;
            }
            Message::Frame(_msg) => {
                unreachable!();
            }
        }
    }

    Ok(())
}
