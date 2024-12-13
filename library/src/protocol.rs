use crate::game::elements::system::System;
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Login {
    pub nickname: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MyVector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ShipState {
    pub throttle_up: bool,
    pub direction: MyVector3,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum PlayerAction {
    Login(Login),
    ShipState(ShipState),
}

#[derive(Serialize, Deserialize)]
pub struct PlayerInfo {
    pub coords: Vector3<f32>,
}

#[derive(Serialize, Deserialize)]
pub struct AuthInfo {
    pub(crate) success: bool,
    pub(crate) message: String,
}

#[derive(Serialize, Deserialize)]
pub enum ServerInfo {
    Player(PlayerInfo),
    System(System),
    PlayersInSystem(Vec<PlayerInfo>),
}

#[derive(Serialize, Deserialize)]
pub struct NextMessage {
    pub next_message_type: String,
}
