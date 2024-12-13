use nalgebra::Vector3;
use scilib::coordinate::{cartesian, spherical};
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

    pub fn translate_from_local_delta(&mut self, local_delta: &SystemCoords) {
        let global_sph =
            spherical::Spherical::from_degree(self.distance as f64, self.theta, self.phi);

        let mut global_car = cartesian::Cartesian::from_coord(global_sph);

        global_car.x += local_delta.x;
        global_car.y += local_delta.y;
        global_car.z += local_delta.z;

        let new_global_sph = spherical::Spherical::from_coord(global_car);

        *self = GalacticCoords {
            distance: new_global_sph.r,
            phi: new_global_sph.phi,
            theta: new_global_sph.theta,
        }
    }
}

pub type SystemCoords = Vector3<f64>;
