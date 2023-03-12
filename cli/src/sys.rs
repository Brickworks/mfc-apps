use log::info;
use std::path::PathBuf;
use toml::Value;
use control_apps::control_mngr::ControlMngr;

pub fn init_altctrl(ctrl_config: &PathBuf) -> ControlMngr {
    info!(
        "Initializing altitude controller from {}",
        ctrl_config.display()
    );
    let config = std::fs::read_to_string(ctrl_config)
        .unwrap()
        .as_str()
        .parse::<Value>()
        .unwrap();
    ControlMngr::new(config)
}
