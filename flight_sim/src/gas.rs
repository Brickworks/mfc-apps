// ----------------------------------------------------------------------------
// Gas
// ---
// Ideal gas equations. This model assumes that temperature and pressure are
// known, and that volume is not constrained. As such, only a gas's species,
// mass, temperature, and pressure can be set explicitly; volume and density
// are read-only derived attributes.
// ----------------------------------------------------------------------------

use std::fmt;

const STANDARD_TEMPERATURE: f32 = 273.15; // [K]
const STANDARD_PRESSURE: f32 = 101325.0; // [Pa]
const BOLTZMANN_CONSTANT: f32 = 1.38e-23_f32; // [J/K]
const AVOGADRO_CONSTANT: f32 = 6.022e+23_f32; // [1/mol]
const R: f32 = BOLTZMANN_CONSTANT * AVOGADRO_CONSTANT; //[J/K-mol] Ideal gas constant

#[derive(Copy, Clone)]
pub enum GasSpecies {
    // Species of gas with a known molar mass (kg/mol)
    Air,
    He,
    Helium,
    H2,
    Hydrogen,
    N2,
    Nitrogen,
    O2,
    Oxygen,
    Ar,
    Argon,
    CO2,
    CarbonDioxide,
    Ne,
    Neon,
    Kr,
    Krypton,
    Xe,
    Xenon,
    CH4,
    Methane,
}

impl fmt::Display for GasSpecies {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            GasSpecies::Air => write!(f, "Air"),
            GasSpecies::He => write!(f, "Helium"),
            GasSpecies::Helium => write!(f, "Helium"),
            GasSpecies::H2 => write!(f, "Hydrogen"),
            GasSpecies::Hydrogen => write!(f, "Hydrogen"),
            GasSpecies::N2 => write!(f, "Nitrogen"),
            GasSpecies::Nitrogen => write!(f, "Nitrogen"),
            GasSpecies::O2 => write!(f, "Oxygen"),
            GasSpecies::Oxygen => write!(f, "Oxygen"),
            GasSpecies::Ar => write!(f, "Argon"),
            GasSpecies::Argon => write!(f, "Argon"),
            GasSpecies::CO2 => write!(f, "Carbon Dioxide"),
            GasSpecies::CarbonDioxide => write!(f, "Carbon Dioxide"),
            GasSpecies::Ne => write!(f, "Neon"),
            GasSpecies::Neon => write!(f, "Neon"),
            GasSpecies::Kr => write!(f, "Krypton"),
            GasSpecies::Krypton => write!(f, "Krypton"),
            GasSpecies::Xe => write!(f, "Xenon"),
            GasSpecies::Xenon => write!(f, "Xenon"),
            GasSpecies::CH4 => write!(f, "Methane"),
            GasSpecies::Methane => write!(f, "Methane"),
        }
    }
}

#[derive(Copy, Clone)]
pub struct GasVolume {
    // A finite amount of a particular gas
    species: GasSpecies, // type of gas
    mass: f32,           // [kg] amount of gas in the volume
    temperature: f32,    // [K] temperature
    pressure: f32,       // [Pa] pressure
    molar_mass: f32,     // [kg/mol] molar mass a.k.a. molecular weight
    volume: f32,         // [m^3] volume
}

impl fmt::Display for GasVolume {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:}: {:} kg | {:} K | {:} Pa | {:} m^3",
            self.species, self.mass, self.temperature, self.pressure, self.volume,
        )
    }
}

impl GasVolume {
    pub fn new(species: GasSpecies, mass: f32) -> Self {
        // Create a new gas volume as a finite amount of mass (kg) of a
        // particular species of gas. Gas is initialized at standard
        // temperature and pressure.
        // --- -------
        // Key Species
        // --- -------
        // He  Helium
        // H2  Hydrogen
        // N2  Nitrogen
        // O2  Oxygen
        // Ar  Argon
        // CO2 Carbon Dioxide
        // Ne  Neon
        // Kr  Krypton
        // Xe  Xenon
        // CH4 Methane
        GasVolume {
            species,
            mass,
            temperature: STANDARD_TEMPERATURE,
            pressure: STANDARD_PRESSURE,
            molar_mass: molar_mass(species),
            volume: volume(
                STANDARD_TEMPERATURE,
                STANDARD_PRESSURE,
                mass,
                molar_mass(species),
            ),
        }
    }

    pub fn set_temperature(&mut self, new_temperature: f32) {
        // set the temperature (K) of the GasVolume
        self.temperature = new_temperature;
        self.update_volume();
    }

    pub fn set_pressure(&mut self, new_pressure: f32) {
        // set the pressure (Pa) of the GasVolume
        self.pressure = new_pressure;
        self.update_volume();
    }

    pub fn set_mass(&mut self, new_mass: f32) {
        // set the mass (kg) of the GasVolume
        self.mass = new_mass;
        self.update_volume();
    }

    pub fn set_mass_from_volume(&mut self) {
        // set the mass (kg) based on the current volume (m^3),
        // density (kg/m^3), and molar mass (kg/mol)
        self.mass = self.volume * (self.molar_mass / R) * (self.pressure / self.temperature);
        self.update_volume();
    }

    pub fn temperature(self) -> f32 {
        // temperature (K)
        return self.temperature;
    }

    pub fn pressure(self) -> f32 {
        // pressure (Pa)
        return self.pressure;
    }

    pub fn mass(self) -> f32 {
        // mass (kg)
        return self.mass;
    }

    pub fn volume(&mut self) -> f32 {
        // volume (m^3)
        self.update_volume();
        return self.volume;
    }

    fn update_volume(&mut self) {
        self.volume = volume(self.temperature, self.pressure, self.mass, self.molar_mass);
    }

    pub fn density(self) -> f32 {
        // density (kg/m^3)
        return density(self.temperature, self.pressure, self.molar_mass);
    }
}

pub fn volume(temperature: f32, pressure: f32, mass: f32, molar_mass: f32) -> f32 {
    // Volume (m^3) of an ideal gas from its temperature (K), pressure (Pa),
    // mass (kg) and molar mass (kg/mol).
    return (mass / molar_mass) * R * temperature / pressure; // [m^3]
}

pub fn density(temperature: f32, pressure: f32, molar_mass: f32) -> f32 {
    // Density (kg/m^3) of an ideal gas frorm its temperature (K), pressure (Pa),
    // and molar mass (kg/mol)
    return (molar_mass * pressure) / (R * temperature); // [kg/m^3]
}

pub fn molar_mass(species: GasSpecies) -> f32 {
    // Get the molecular weight (kg/mol) of a dry gas at sea level.
    // Source: US Standard Atmosphere, 1976
    match species {
        GasSpecies::Air => 0.02897,
        GasSpecies::He | GasSpecies::Helium => 0.0040026,
        GasSpecies::H2 | GasSpecies::Hydrogen => 0.00201594,
        GasSpecies::N2 | GasSpecies::Nitrogen => 0.0280134,
        GasSpecies::O2 | GasSpecies::Oxygen => 0.0319988,
        GasSpecies::Ar | GasSpecies::Argon => 0.039948,
        GasSpecies::CO2 | GasSpecies::CarbonDioxide => 0.04400995,
        GasSpecies::Ne | GasSpecies::Neon => 0.020183,
        GasSpecies::Kr | GasSpecies::Krypton => 0.08380,
        GasSpecies::Xe | GasSpecies::Xenon => 0.13130,
        GasSpecies::CH4 | GasSpecies::Methane => 0.01604303,
    }
}
