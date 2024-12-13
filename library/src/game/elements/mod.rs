use scilib::coordinate::{cartesian, spherical};

use super::repr::{GalacticCoords, SystemCoords};

pub mod body;
pub mod player;
pub mod system;

pub fn move_from_local_delta(
    global_coords: &GalacticCoords,
    local_delta: &SystemCoords,
) -> GalacticCoords {
    let global_sph = spherical::Spherical::from_degree(
        global_coords.distance as f64,
        global_coords.theta,
        global_coords.phi,
    );

    let mut global_car = cartesian::Cartesian::from_coord(global_sph);

    global_car.x += local_delta.x;
    global_car.y += local_delta.y;
    global_car.z += local_delta.z;

    let new_global_sph = spherical::Spherical::from_coord(global_car);

    GalacticCoords {
        distance: new_global_sph.r,
        phi: new_global_sph.phi,
        theta: new_global_sph.theta,
    }
}
