pub mod client;
pub mod error;
pub mod game;
pub mod input;
pub mod network;
pub mod protocol;
pub mod server;
pub mod service;

pub type Result<T> = std::result::Result<T, crate::error::Error>;
