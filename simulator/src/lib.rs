mod balloon;
mod force;
mod gas;

pub mod async_sim;
pub mod simulate;

pub struct SimCommands {
    pub vent_flow_percentage: f32,
    pub dump_flow_percentage: f32,
}

#[derive(Default, Copy, Clone)]
pub struct SimOutput {
    pub time_s: f32,
    pub altitude: f32,
    pub ascent_rate: f32,
    pub acceleration: f32,
    pub ballast_mass: f32,
    pub lift_gas_mass: f32,
    pub vent_pwm: f32,
    pub dump_pwm: f32,
}
