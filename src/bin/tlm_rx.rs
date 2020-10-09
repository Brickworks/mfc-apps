extern crate rmp_serde as rmps;

use std::io::{Cursor, Write};
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc::{Receiver, SyncSender};
use std::thread;
use std::time::Duration;

use nng;
use rmp;
use serde;

static UDP_RX_ADDR: ([u8; 4], u16) = ([127, 0, 0, 1], 6666);
static NNG_TX_ADDR: &str = "ipc://nucleus";

#[derive(Debug, PartialEq, serde::Deserialize)]
struct LatLon {
    lat: f32,
    lon: f32,
}

/// Relays UDP msgpack messages to the IPC sender
/// Not expected to terminate
fn eth_rx_loop(thread_tx: SyncSender<Vec<u8>>) {
    let mut buf = [0; 256];
    assert!(buf[255] == 0);


    let socket = match UdpSocket::bind(SocketAddr::from(UDP_RX_ADDR)) {
        Ok(v) => v,
        Err(e) => {
            println!("Creating socket error: {:?}", e);
            return ();
        }
    };


    // Loop on listening for UDP packets, and relaying them over to the IPC sender
    loop {
        let (size, _sender_addr) = match socket.recv_from(&mut buf[..]) {
            Ok(v) => {
                println!("Received {} byes", v.0);
                v
            },
            Err(e) => {
                println! {"Error receiving message: {:?}", e};
                continue;
            }
        };

        
        match thread_tx.send(buf[..size].to_vec()) {
            Ok(_) => (),
            Err(e) => println!("Error sending eth rx intrapc: {:?}", e),
        };
    }
}

/// Relays msgpack CAN messages to IPC sender thread
/// Not expected to terminate
fn can_rx_loop(_thread_tx: SyncSender<Vec<u8>>) {
    loop {}
}

/// Construct a NNG message with the given topic
/// TODO: Move to common code
fn fmt_nng_msg(topic: &str, body: &[u8]) -> Vec<u8> {
    [topic.as_bytes(), ":".as_bytes(), body].concat()
}

/// Receives intra-thread messages to publish to a NNG socket for IPC
/// Not expected to terminate
fn ipc_tx_loop(thread_rx: Receiver<Vec<u8>>) {
    // TODO move initialization of sockets to init function, moving into thread
    let s = nng::Socket::new(nng::Protocol::Pub0).unwrap();
    s.listen(NNG_TX_ADDR).unwrap();
    let mut out_msg = nng::Message::new().unwrap();
    // TODO use contexts and spawn threads

    loop {
        let buf = match thread_rx.recv() {
            Ok(v) => v,
            Err(_) => {
                println!("Channel disconnected");
                break;
            }
        };

        // grab extension code and size of data
        let ext_meta = rmp::decode::read_ext_meta(&mut Cursor::new(&buf)).unwrap();

        // chop off extension, this might be janky? TODO find a better way
        let data = buf[(buf.len() - (ext_meta.size as usize))..].to_vec();

        // publish message on nng
        match out_msg.write_all(&fmt_nng_msg("hello", data.as_slice())) {
            Ok(_) => (),
            Err(e) => println!("Error sending nng msg: {:?}", e),
        };
    }
}

fn main() {
    // TODO move setup of sockets and all to an init function, moving the objects into the threads
    let (eth_thread_sender, thread_rx) = std::sync::mpsc::sync_channel(1);
    let can_thread_sender = eth_thread_sender.clone();

    let eth_rx_handler = thread::spawn(move || { eth_rx_loop(eth_thread_sender) });
    let can_rx_hander = thread::spawn(|| { can_rx_loop(can_thread_sender) });
    let ipc_tx_handler = thread::spawn(|| { ipc_tx_loop(thread_rx) });

    match eth_rx_handler.join() {
        Ok(_) => println!("eth_rx Shit worked out"),
        Err(e) => println!("eth_rx Error: {:?}", e)
    }
    can_rx_hander.join().unwrap();

    ipc_tx_handler.join().unwrap();
}
