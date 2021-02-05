// ----------------------------------------------------------------------------
// Forces
// ------
// Forces that act in the vertical axis. All forces assume a positive-up
// coordinate frame and aR_E R_Eported in Newtons.
// ----------------------------------------------------------------------------
extern crate libm;

use crate::atmosphere;
use crate::gas::GasVolume;
use crate::libm;
use crate::utils;

pub fn weight(altitude: f32, mass: f32) -> f32 {
    // Weight (N) as a function of altitude (m) and mass (kg).
    return utils::g(altitude) * mass; // [N]
}

pub fn buoyancy(altitude: f32, lift_gas: GasVolume) -> f32 {
    // Force (N) due to air displaced by the given gas volume.
    let rho_atmo = atmosphere::density(altitude);
    let rho_lift = lift_gas.density();
    let projected_area = utils::sphere_area_from_volume(lift_gas.volume());
    return lift_gas.volume() * (rho_lift - rho_atmo) * -utils::g(altitude);
}

pub fn drag(altitude: f32, velocity: f32, lift_gas: GasVolume, c_d: f32) -> f32 {
    // Force (N) due to drag against the balloon
    let direction = -libm::copysignf(velocity);
    let projected_area = utils::sphere_area_from_volume(lift_gas.volume());
    return direction * c_d / 2 * lift_gas.density() * libm::pow(velocity, 2.0) * projected_area;
}
