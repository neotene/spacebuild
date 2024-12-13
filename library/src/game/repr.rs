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
    pub fn new(phi: Angle, theta: Angle, distance: Distance) -> Self {
        GalacticCoords {
            theta,
            phi,
            distance,
        }
    }
}

pub type SystemCoords = Vector3<f64>;
