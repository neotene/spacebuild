use crate::game::{elements::system::System, repr::GalacticCoords};
use nalgebra::Vector3;
use rand::Rng;
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

// pub fn gen_system() -> System {
//     let mut rng = rand::thread_rng();
//     let angle_1 = rng.gen_range(0..15000) as f64 / 10000.;
//     let angle_2 = rng.gen_range(0..15000) as f64 / 10000.;
//     let distance = rng.gen_range(0.0..10000000000.);

//     System::new(GalacticCoords::new(angle_1, angle_2, distance))
// }
