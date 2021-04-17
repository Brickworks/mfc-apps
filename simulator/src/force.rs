// ----------------------------------------------------------------------------
// Forces
// ------
// Forces that act in the vertical axis. All forces assume a positive-up
// coordinate frame and aR_E R_Eported in Newtons.
// ----------------------------------------------------------------------------
extern crate libm;

use crate::gas;
use std::f32::consts::PI;

const STANDARD_G: f32 = 9.80665; // [m/s^2] standard gravitational acceleration
const EARTH_RADIUS_M: f32 = 6371007.2; // [m] mean radius of Earth

fn g(altitude: f32) -> f32 {
    // Acceleration (m/s^2) from gravity at an altitude (m) above mean sea level.
    return -STANDARD_G * (EARTH_RADIUS_M / (EARTH_RADIUS_M + altitude)); // [m/s^2]
}

fn weight(altitude: f32, mass: f32) -> f32 {
    // Weight (N) as a function of altitude (m) and mass (kg).
    return g(altitude) * mass; // [N]
}

fn buoyancy(altitude: f32, atmo: gas::Atmosphere, lift_gas: gas::GasVolume) -> f32 {
    // Force (N) due to air displaced by the given gas volume.
    let rho_atmo = atmo.density();
    let rho_lift = lift_gas.density();
    return lift_gas.volume() * (rho_lift - rho_atmo) * g(altitude);
}

fn drag(atmo: gas::Atmosphere, velocity: f32, projected_area: f32, c_d: f32) -> f32 {
    // Force (N) due to drag against the balloon
    let direction = -libm::copysignf(1.0, velocity);
    return direction * c_d / 2.0 * atmo.density() * libm::powf(velocity, 2.0) * projected_area;
}

pub fn net_force(
    altitude: f32,
    velocity: f32,
    atmo: gas::Atmosphere,
    lift_gas: gas::GasVolume,
    projected_area: f32,
    c_d: f32,
    total_dry_mass: f32,
) -> f32 {
    // [N]
    let weight_force = weight(altitude, total_dry_mass);
    let buoyancy_force = buoyancy(altitude, atmo, lift_gas);
    let drag_force = drag(atmo, velocity, projected_area, c_d);
    return weight_force + buoyancy_force + drag_force;
}

pub fn gross_lift(atmo: gas::Atmosphere, lift_gas: gas::GasVolume) -> f32 {
    // [kg]
    let rho_atmo = atmo.density();
    let rho_lift = lift_gas.density();
    return lift_gas.volume() * (rho_lift - rho_atmo);
}

pub fn free_lift(atmo: gas::Atmosphere, lift_gas: gas::GasVolume, total_dry_mass: f32) -> f32 {
    // [kg]
    return gross_lift(atmo, lift_gas) - total_dry_mass;
}

pub fn sphere_area_from_volume(volume: f32) -> f32 {
    // Get the projected area (m^2) of a sphere with a given volume (m^3)
    return libm::powf(libm::powf(volume / (PI * (4.0 / 3.0)), 1.0 / 3.0), 2.0) * PI;
}