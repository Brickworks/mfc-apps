// ----------------------------------------------------------------------------
// Atmosphere
// ----------
// Approximate atmospheric temperature and pressure as a function of altitude.
// Based on the US Standard Atmosphere, 1976.
// Reference:
//  https://apps.dtic.mil/dtic/tr/fulltext/u2/a035728.pdf
//  https://www.translatorscafe.com/unit-converter/en-US/calculator/altitude
//  https://www.grc.nasa.gov/WWW/K-12/airplane/atmosmet.html
// ----------------------------------------------------------------------------

extern crate libm;

use crate::libm;
use crate::utils;
use crate::gas;

pub fn temperature(altitude: f32) -> f32 {
    // Temperature (K) of the atmosphere at a given altitude (m).
    // Only valid for altitudes below 85,000 meters.
    let atmo_temp_c = match altitude {
        altitude < 11000.0 => (15.04 - 0.00649 * altitude),
        altitude >= 11000.0 && altitude < 25000.0 => -56.46,
        altitude >= 25000.0 && altitude < 85000.0 => (-131.21 + 0.00299 * altitude)
        _ => error!("Altitude {:}m is outside of the accepted range! Must be 0-85,000m", altitude);
    } // temperature in Celsius
    return utils::temperature_c2k(atmo_temp_c)
}

pub fn pressure(altitude: f32) -> f32 {
    // Pressure (Pa) of the atmosphere at a given altitude (m).
    // Only valid for altitudes below 85,000 meters.
    let atmo_pres_kPa = match altitude {
        altitude < 11000.0 => (101.29 * libm::pow((temperature(altitude) / 288.08),5.256)),
        altitude >= 11000.0 && altitude < 25000.0 => (22.65 * libm::exp(1.73 - 0.000157 * altitude)),
        altitude >= 25000.0 && altitude < 85000.0 => (2.488 * libm::pow((temperature(altitude) / 216.6), -11.388))
        _ => error!("Altitude {:}m is outside of the accepted range! Must be 0-85,000m", altitude);
    } // pressure in kPa
    return atmo_pres_kPa * 1000.0
}

pub fn density(altitude: f32) -> f32 {
    // Density (kg/m^3) of the atmosphere at a given altitude (m).
    // Only valid for altitudes below 85,000 meters.
    return gas::density(
        temperature(altitude),
        pressure(altitude),
        gas::molar_mass(String::from_str("air"))
    )
}
