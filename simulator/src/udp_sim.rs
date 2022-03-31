use crate::async_sim::{AsyncSim, Rate};

use std::net::UdpSocket;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::{SimCommands, SimOutput};
use serde_json;
use toml;

fn transmit_sim_output(sim: Arc<Mutex<AsyncSim>>) {
    let out_socket = UdpSocket::bind("localhost:1337").unwrap();

    let mut rate_sleeper = Rate::new(10.0);
    loop {
        let sim_output = sim.lock().unwrap().get_sim_output();

        let serialized_output = serde_json::to_string(&sim_output).unwrap();

        out_socket
            .send_to(serialized_output.as_bytes(), "localhost:1338")
            .unwrap();

        // sleep
        rate_sleeper.sleep();
    }
}

fn receive_commands(sim: Arc<Mutex<AsyncSim>>) {
    let in_socket = UdpSocket::bind("localhost:1339").unwrap();

    let mut buf = [0; 1024];

    loop {
        let num_bytes_rx = in_socket.recv(&mut buf).unwrap();

        let deserialized_msg: SimCommands = serde_json::from_slice(&buf[..num_bytes_rx]).unwrap();
        println!("{:#?}", deserialized_msg);
    }
}

pub fn run(sim_cfg_path: &PathBuf, sim_output_path: &PathBuf) {
    let sim_cfg = std::fs::read_to_string(sim_cfg_path)
        .unwrap()
        .as_str()
        .parse::<toml::Value>()
        .unwrap();
    let sim = Arc::new(Mutex::new(AsyncSim::new(sim_cfg, sim_output_path.clone())));

    sim.lock().unwrap().start();

    let transmit_sim_clone = sim.clone();
    let receive_sim_clone = sim.clone();
    std::thread::spawn(move || transmit_sim_output(transmit_sim_clone));
    std::thread::spawn(move || receive_commands(receive_sim_clone));
}
