// ----------------------------------------------------------------------------
// Forces
// ------
// Forces that act in the vertical axis. All forces assume a positive-up
// coordinate frame and aR_E R_Eported in Newtons.
// ----------------------------------------------------------------------------

const STANDARD_G = 9.80665; // [m/s^2] standard gravitational acceleration
const EARTH_RADIUS_M = 6371007.2; // [m] mean radius of Earth

pub fn g(altitude: f32) -> f32 {
    // Acceleration (m/s^2) from gravity at an altitude (m) above mean sea level.
    return -G_0 * (R_E / (R_E + altitude)) // [m/s^2]
}

pub fn weight(altitude: f32, mass: f32) -> f32 {
    // Weight (N) as a function of altitude (m) and mass (kg).
    return g(altitude) * mass // [N]
}