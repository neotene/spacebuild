use nalgebra::Vector3;
use scilib::coordinate::{cartesian, spherical};
use serde::{Deserialize, Serialize};

use super::galaxy::Galactic;

pub type Angle = f64; // radian
pub type Distance = f64; // cm
pub type Speed = f64;
pub type Direction = Vector3<f64>;
pub type LocalCoords = Vector3<f64>;

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub struct GlobalCoords {
    pub phi: Angle,
    pub theta: Angle,
    pub distance: Distance,
}

impl GlobalCoords {
    pub fn new(phi: Angle, theta: Angle, distance: Distance) -> Self {
        assert!(!phi.is_nan());
        assert!(!theta.is_nan());
        assert!(!distance.is_nan());
        GlobalCoords {
            phi,
            theta,
            distance,
        }
    }

    pub fn get_global_car(&self) -> cartesian::Cartesian {
        assert!(!self.phi.is_nan());
        assert!(!self.theta.is_nan());
        assert!(!self.distance.is_nan());
        let global_sph =
            spherical::Spherical::from_degree(self.distance as f64, self.theta, self.phi);

        cartesian::Cartesian::from_coord(global_sph)
    }

    pub fn get_local_from_element(&self, element: &Galactic) -> LocalCoords {
        assert!(!self.phi.is_nan());
        assert!(!self.theta.is_nan());
        assert!(!self.distance.is_nan());
        let diff = self.get_global_car() - element.coords.get_global_car();
        LocalCoords::new(diff.x, diff.y, diff.z)
    }

    pub fn translate_from_local_delta(&mut self, local_delta: &LocalCoords) {
        if local_delta.magnitude() == 0. {
            return;
        }
        assert!(!self.phi.is_nan());
        assert!(!self.theta.is_nan());
        assert!(!self.distance.is_nan());
        let mut global_car = self.get_global_car();

        global_car.x += local_delta.x;
        global_car.y += local_delta.y;
        global_car.z += local_delta.z;

        let new_global_sph = spherical::Spherical::from_coord(global_car);

        *self = GlobalCoords {
            distance: new_global_sph.r,
            phi: new_global_sph.phi,
            theta: new_global_sph.theta,
        }
    }
}
