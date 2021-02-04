// ----------------------------------------------------------------------------
// Utils
// -----
// Helper functions with common uses.
// ----------------------------------------------------------------------------

pub fn temperature_c2k(temp_celsius: f32) -> f32 {
    // Convert temperature from Kelvin to Celsius
    return temp_celsius + 273.1
}

pub fn temperature_k2c(temp_kelvin: f32) -> f32 {
    // Convert temperature from Kelvin to Celsius
    return temp_kelvin - 273.1
}