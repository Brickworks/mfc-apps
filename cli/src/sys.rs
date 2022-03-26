use log::info;
use std::path::PathBuf;
use toml::Value;
use control_apps::control_mngr::ControlMngr;

pub fn init_altctrl(ctrl_config: &PathBuf) -> ControlMngr {
    let config = std::fs::read_to_string(ctrl_config)
        .unwrap()
        .as_str()
        .parse::<Value>()
        .unwrap();

    info!(
        "Setting up altitude controller with following config: \n{}",
        config
    );
    ControlMngr::new(config)
}
