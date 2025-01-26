use rustls_pki_types::InvalidDnsNameError;
use tokio::io;
use tokio_tungstenite::tungstenite;

use crate::Id;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Error")]
    Error,
    #[error("TLS task encountered an error: {0}")]
    CriticalFromTls(String),
    #[error("HTTP task encountered an error: {0}")]
    CriticalFromHttp(String),
    #[error("WS task encountered an error: {0}")]
    CriticalFromWs(String),
    #[error("Could not get last id in a table: {0}")]
    DbLastIdError(sqlx::Error),
    #[error("JSON: can't deserialize {0}: {1}")]
    DeserializeError(String, serde_json::Error),
    #[error("SqlDb: can't insert ({0}): {1}")]
    SqlDbInsertError(String, sqlx::Error),
    #[error("Uuid not found in db: {0}")]
    DbUuidNotFound(Id),
    #[error("Gravity center not found")]
    GravityCenterNotFound,
    #[error("Invalid nickname")]
    InvalidNickname,
    #[error("Can't load a certificate: {0}")]
    CertLoadError(rustls_pki_types::pem::Error),
    #[error("Could not TLS handshake: {0}")]
    TlsHandshakeError(InvalidDnsNameError),
    #[error("data not upgrade to TLS: {0}")]
    CouldNotUpgradeToTls(io::Error),
    #[error("Can't create table {0}: {1}")]
    DbCreateTableError(String, sqlx::Error),
    #[error("Can't join in '{0}' and '{1}' with where clause '{2}': {3}")]
    DbSelectFromJoinedIdsError(String, String, String, sqlx::Error),
    #[error("Can't select in '{0}' with where clause '{1}': {2}")]
    DbSelectFromWhereError(String, String, sqlx::Error),
    #[error("DB file creation error {0}")]
    DbFileCreationError(std::io::Error),
    #[error("DB invalid ID: {0}")]
    DbInvalidUuidError(Id),
    #[error("CRITICAL: found several ({0}) players with same nickname")]
    DbLoadPlayerByNicknameFoundTooMany(usize),
    #[error("Can't load player: nickname not found")]
    DbLoadPlayerByNicknameNotFound,
    #[error("Can't load player by nickname: {0}")]
    DbLoadPlayerByNicknameQueryError(sqlx::Error),
    #[error("Error while loading from SqliteRow: {0}")]
    DbLoadError(sqlx::Error),
    #[error("Can't open DB {0}: {1}")]
    DbOpenError(String, sqlx::Error),
    #[error("Error while trying to deserialize authentication response from server {0} {1}")]
    DeserializeAuthenticationResponseError(serde_json::Error, String),
    #[error("Failed to serialize login")]
    FailedToSerializeLogin(serde_json::Error),
    #[error("Can't load a key: {0}")]
    KeyLoadError(rustls_pki_types::pem::Error),
    #[error("Player already authenticated")]
    PlayerAlreadyAuthenticated,
    #[error("Player deserialization error: {0}")]
    PlayerDeserializationError(serde_json::Error),
    #[error("Could not connect through TCP: {0}")]
    TcpCouldNotConnect(io::Error),
    #[error("Can't build tls config: {0}")]
    TlsConfigBuildError(rustls::Error),
    #[error("Websocket send: {0}")]
    WsCantSend(tungstenite::Error),
    #[error("Websocket read: {0}")]
    WsCantRead(tungstenite::Error),
    #[error("Unexpected response from server: {0}")]
    UnexpectedResponse(String),
    #[error("Bad UUID in \"{0}\"")]
    BadUuidError(String),
    #[error("Cannot close connection gracefully: {0}")]
    GracefulCloseError(tungstenite::Error),
}
