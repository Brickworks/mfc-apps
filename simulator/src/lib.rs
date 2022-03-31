use serde::{Deserialize, Serialize};

mod balloon;
mod force;
mod gas;

pub mod async_sim;
pub mod simulate;
pub mod udp_sim;

#[derive(Debug, Serialize, Deserialize)]
pub struct SimCommands {
    pub vent_flow_percentage: f32,
    pub dump_flow_percentage: f32,
}

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct SimOutput {
    pub time_s: f32,
    pub altitude: f32,
    pub ascent_rate: f32,
    pub acceleration: f32,
    pub ballast_mass: f32,
    pub lift_gas_mass: f32,
    pub vent_pwm: f32,
    pub dump_pwm: f32,
    pub gross_lift: f32,
    pub free_lift: f32,
    pub atmo_temp: f32,
    pub atmo_pres: f32,
}
