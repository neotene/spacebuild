use rustls_pki_types::InvalidDnsNameError;
use tokio::io;
use tokio_tungstenite::tungstenite;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Error")]
    Error,
    #[error("Can't load a certificate: {0}")]
    CertLoadError(rustls_pki_types::pem::Error),
    #[error("Could not TLS handshake: {0}")]
    TlsHandshakeError(InvalidDnsNameError),
    #[error("data not upgrade to TLS: {0}")]
    CouldNotUpgradeToTls(io::Error),
    #[error("Can't create player table: {0}")]
    DbCreatePlayerTableError(sqlx::Error),
    #[error("DB file creation error {0}")]
    DbFileCreationError(std::io::Error),
    #[error("DB invalid UUID")]
    DbInvalidUuidError(uuid::Error),
    #[error("CRITICAL: found several players with same nickname")]
    DbLoadPlayerByNicknameFoundTooMany,
    #[error("Can't load player: nickname not found")]
    DbLoadPlayerByNicknameNotFound,
    #[error("Can't load player by nickname: {0}")]
    DbLoadPlayerByNicknameQueryError(sqlx::Error),
    #[error("Error while loading systems: {0}")]
    DbLoadSystemsError(sqlx::Error),
    #[error("Can't open DB {0}: {1}")]
    DbOpenError(String, sqlx::Error),
    #[error("Can't sync systems to DB: {0}")]
    DbSyncSystemsToDbError(sqlx::Error),
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
    WSCantSend(tungstenite::Error),
    #[error("Unexpected response from server: {0}")]
    UnexpectedResponse(String),
    #[error("Bad UUID: {0} in \"{1}\"")]
    BadUuidError(uuid::Error, String),
    #[error("Cannot close connection gracefully: {0}")]
    GracefulCloseError(tungstenite::Error),
}
