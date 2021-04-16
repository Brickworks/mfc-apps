// ----------------------------------------------------------------------------
// Balloon
// -------
// 
// ----------------------------------------------------------------------------

extern crate libm;
use std::f32::consts::PI;
use crate::gas;

#[derive(Copy, Clone)]
pub enum BalloonType {
    // balloon part numbers
    HAB_800,
    HAB_1200,
    HAB_1500,
    HAB_2000,
    HAB_3000,
}

#[derive(Copy, Clone)]
pub struct Balloon {
    part_no: BalloonType,
    pub lift_gas: gas::GasVolume,
    pub mass: f32, // balloon mass
    pub max_volume: f32, // burst above this volume
    pub c_d: f32, // balloon approx drag coefficient
    pub recommended_free_lift: f32, // recommended free lift at launch
    pub intact: bool, // whether or not it has burst
}

impl Balloon {
    pub fn new(part_no: BalloonType, lift_gas: gas::GasVolume) -> Self {
        match part_no {
            BalloonType::HAB_800 => {
                Balloon {
                    part_no,
                    lift_gas,
                    mass: 0.8,
                    max_volume: volume_from_diameter(7.0),
                    c_d: 0.3,
                    recommended_free_lift: 0.970,
                    intact: true,
                }
            },
            BalloonType::HAB_1200 => {
                Balloon {
                    part_no,
                    lift_gas,
                    mass: 1.2,
                    max_volume: volume_from_diameter(8.63),
                    c_d: 0.25,
                    recommended_free_lift: 1.19,
                    intact: true,
                }
            },
            BalloonType::HAB_1500 => {
                Balloon {
                    part_no,
                    lift_gas,
                    mass: 1.5,
                    max_volume: volume_from_diameter(9.44),
                    c_d: 0.25,
                    recommended_free_lift: 1.28,
                    intact: true,
                }
            },
            BalloonType::HAB_2000 => {
                Balloon {
                    part_no,
                    lift_gas,
                    mass: 2.0,
                    max_volume: volume_from_diameter(10.54),
                    c_d: 0.25,
                    recommended_free_lift: 1.42,
                    intact: true,
                }
            },
            BalloonType::HAB_3000 => {
                Balloon {
                    part_no,
                    lift_gas,
                    mass: 3.0,
                    max_volume: volume_from_diameter(13.0),
                    c_d: 0.25,
                    recommended_free_lift: 1.67,
                    intact: true,
                }
            },
        }
    }

    fn burst(&mut self) {
        // Change balloon attributes if it has burst
        self.intact = false;
        self.c_d = 0.0;
        // mass is conserved, it just no longer holds gas
    }

    pub fn check_burst_condition(&mut self) {
        if self.lift_gas.volume() > self.max_volume {
            self.burst();
        }
    }
}

fn volume_from_diameter(diameter: f32) -> f32 {
    // spherical volume given its diameter
    return (4.0 / 3.0) * PI * libm::powf(diameter / 2.0, 3.0);
}
