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
    pub altitude: f32,
    pub ascent_rate: f32,
    pub ballast_mass: f32,
    pub time_s: f32,
}
