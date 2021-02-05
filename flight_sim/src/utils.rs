// ----------------------------------------------------------------------------
// Utils
// -----
// Helper functions with common uses.
// ----------------------------------------------------------------------------

extern crate libm;

use std::f32::consts::PI;

const STANDARD_G: f32 = 9.80665; // [m/s^2] standard gravitational acceleration
const EARTH_RADIUS_M: f32 = 6371007.2; // [m] mean radius of Earth

pub fn g(altitude: f32) -> f32 {
    // Acceleration (m/s^2) from gravity at an altitude (m) above mean sea level.
    return -STANDARD_G * (EARTH_RADIUS_M / (EARTH_RADIUS_M + altitude)); // [m/s^2]
}

pub fn temperature_c2k(temp_celsius: f32) -> f32 {
    // Convert temperature from Kelvin to Celsius
    return temp_celsius + 273.1;
}

pub fn temperature_k2c(temp_kelvin: f32) -> f32 {
    // Convert temperature from Kelvin to Celsius
    return temp_kelvin - 273.1;
}

pub fn sphere_area_from_volume(volume: f32) -> f32 {
    // Get the projected area (m^2) of a sphere with a given volume (m^3)
    return libm::powf(libm::powf(volume / (PI * (4.0 / 3.0)), 1.0 / 3.0), 2.0) * PI;
}
