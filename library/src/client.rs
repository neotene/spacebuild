use std::str::FromStr;

use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;

use tokio_rustls::client::TlsStream;
use tokio_tungstenite::tungstenite::{client::IntoClientRequest, Message};
use uuid::Uuid;

use crate::error::Error;
use crate::network::tcp::{connect, ClientStream};
use crate::network::tls::ClientPki;

use crate::{
    protocol::{Login, LoginInfo, PlayerAction},
    Result,
};

enum WebSocketStream {
    Tls(tokio_tungstenite::WebSocketStream<TlsStream<TcpStream>>),
    Tcp(tokio_tungstenite::WebSocketStream<TcpStream>),
}

impl WebSocketStream {
    async fn send(&mut self, login_json: Message) -> Result<()> {
        match self {
            Self::Tcp(stream) => {
                stream
                    .send(login_json)
                    .await
                    .map_err(|err| Error::WSCantSend(err))?;
            }
            Self::Tls(stream) => {
                stream
                    .send(login_json)
                    .await
                    .map_err(|err| Error::WSCantSend(err))?;
            }
        }
        Ok(())
    }

    async fn next(&mut self) -> Result<Message> {
        let response = match self {
            Self::Tls(stream) => stream.next().await.unwrap().map_err(|_err| Error::Error)?,
            Self::Tcp(stream) => stream.next().await.unwrap().map_err(|_err| Error::Error)?,
        };
        Ok(response)
    }
}

pub struct Client {
    stream: WebSocketStream,
}

impl Client {
    pub async fn terminate(&mut self) -> Result<()> {
        match &mut self.stream {
            WebSocketStream::Tcp(stream) => {
                stream
                    .close(None)
                    .await
                    .map_err(|err| Error::GracefulCloseError(err))?;
            }
            WebSocketStream::Tls(stream) => {
                stream
                    .close(None)
                    .await
                    .map_err(|err| Error::GracefulCloseError(err))?;
            }
        }
        Ok(())
    }

    pub async fn connect(addr: &str, pki: Option<ClientPki<'_>>) -> Result<Client> {
        let mut host = if let None = pki {
            "wss://".to_string()
        } else {
            "wss://".to_string()
        };

        host += addr;

        let stream = connect(format!("{}", addr).as_str(), pki).await?;

        let request = host.into_client_request().unwrap();

        let (stream, _response) = match stream {
            ClientStream::Tcp(stream) => {
                match tokio_tungstenite::client_async(request, stream).await {
                    Ok((stream, response)) => (WebSocketStream::Tcp(stream), response),
                    Err(_err) => {
                        return Err(Error::Error);
                    }
                }
            }
            ClientStream::Tls(stream) => {
                match tokio_tungstenite::client_async(request, stream).await {
                    Ok((stream, response)) => (WebSocketStream::Tls(stream), response),
                    Err(_err) => {
                        return Err(Error::Error);
                    }
                }
            }
        };

        Ok(Client { stream })
    }

    pub async fn login(&mut self, nickname: &str) -> Result<Uuid> {
        let login = PlayerAction::Login(Login {
            nickname: nickname.to_string(),
        });
        let login_json =
            serde_json::to_string(&login).map_err(|err| Error::FailedToSerializeLogin(err))?;

        self.stream.send(Message::Text(login_json)).await?;

        let response = self.stream.next().await?;

        match response {
            Message::Text(response_str) => {
                let login_info: LoginInfo = serde_json::from_str(&response_str).map_err(|err| {
                    Error::DeserializeAuthenticationResponseError(err, response_str)
                })?;

                let uuid = Uuid::from_str(login_info.message.as_str())
                    .map_err(|err| Error::BadUuidError(err, login_info.message))?;

                return Ok(uuid);
            }
            _ => return Err(Error::UnexpectedResponse(format!("{:?}", response))),
        }
    }
}
