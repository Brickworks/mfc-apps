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

use crate::gas;
use crate::libm;
use crate::utils;

pub fn temperature(altitude: f32) -> f32 {
    // Temperature (K) of the atmosphere at a given altitude (m).
    // Only valid for altitudes below 85,000 meters.
    if altitude >= -57.0 && altitude < 11000.0 {
        return utils::temperature_c2k(15.04 - 0.00649 * altitude);
    } else if altitude >= 11000.0 && altitude < 25000.0 {
        return utils::temperature_c2k(-56.46);
    } else if altitude >= 25000.0 && altitude < 85000.0 {
        return utils::temperature_c2k(-131.21 + 0.00299 * altitude);
    } else {
        error!(
            "Altitude {:}m is outside of the accepted range! Must be 0-85,000m",
            altitude
        );
        return 0.0;
    }
}

pub fn pressure(altitude: f32) -> f32 {
    // Pressure (Pa) of the atmosphere at a given altitude (m).
    // Only valid for altitudes below 85,000 meters.
    if altitude >= -57.0 && altitude < 11000.0 {
        return 101.29 * libm::powf(temperature(altitude) / 288.08, 5.256) * 1000.0;
    } else if altitude >= 11000.0 && altitude < 25000.0 {
        return 22.65 * libm::expf(1.73 - 0.000157 * altitude) * 1000.0;
    } else if altitude >= 25000.0 && altitude < 85000.0 {
        return 2.488 * libm::powf(temperature(altitude) / 216.6, -11.388) * 1000.0;
    } else {
        error!(
            "Altitude {:}m is outside of the accepted range! Must be 0-85,000m",
            altitude
        );
        return 0.0;
    }
}

pub fn density(altitude: f32) -> f32 {
    // Density (kg/m^3) of the atmosphere at a given altitude (m).
    // Only valid for altitudes below 85,000 meters.
    return gas::density(
        temperature(altitude),
        pressure(altitude),
        gas::molar_mass(gas::GasSpecies::Air),
    );
}
