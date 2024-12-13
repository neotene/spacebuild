use nalgebra::Vector3;
use serde::{Deserialize, Serialize};

pub type Angle = f64; // radian
pub type Distance = f64; // cm
pub type Speed = f64;
pub type Direction = Vector3<f64>;

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub struct GalacticCoords {
    pub theta: Angle,
    pub phi: Angle,
    pub distance: Distance,
}

impl GalacticCoords {
    pub fn new(angle_1: Angle, angle_2: Angle, distance: Distance) -> Self {
        GalacticCoords {
            theta: angle_1,
            phi: angle_2,
            distance,
        }
    }
}

pub type SystemCoords = Vector3<f64>;
