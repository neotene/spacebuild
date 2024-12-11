use nalgebra::Vector3;
use serde::{Deserialize, Serialize};

pub type Angle = f64; // radian
pub type Distance = u64; // cm
pub type Speed = f64;
pub type Direction = Vector3<f64>;

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub struct Coords {
    pub angle_1: Angle,
    pub angle_2: Angle,
    pub distance: Distance,
}

impl Coords {
    pub fn new(angle_1: Angle, angle_2: Angle, distance: Distance) -> Self {
        Coords {
            angle_1,
            angle_2,
            distance,
        }
    }
}
