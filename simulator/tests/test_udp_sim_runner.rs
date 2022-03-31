//use crate::simulator::udp_simulator;
use simulator::udp_sim;
use simulator::{async_sim, SimOutput};

use std::net::UdpSocket;
use std::path::PathBuf;

use serde_json;

#[test]
fn test_udp_sim() {
    let rx_sock = UdpSocket::bind("localhost:1338").unwrap();

    let cfg_path = PathBuf::from("../support_apps/config/sim_config.toml");
    let out_path = PathBuf::from("./out.txt");

    udp_sim::run(&cfg_path, &out_path);

    let mut buf: [u8; 1024] = [0; 1024];
    let num_bytes = rx_sock.recv(&mut buf).unwrap();

    let deserialized_msg: SimOutput = serde_json::from_slice(&buf[..num_bytes]).unwrap();

    println!("{:#?}", deserialized_msg);

    assert!(true);
}
