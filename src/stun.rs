use std::{collections::HashMap, net::{SocketAddr, UdpSocket}};

use crate::{util::die, messages::UDPMessage};

pub fn spawn_stun(listen: &str) -> ! {
    // Bind on address
    let socket = match UdpSocket::bind(listen) {
        Ok(s) => s,
        Err(err) => die(err),
    };
    // Setup variables
    let mut buffer = [0; 128];
    let mut servers: HashMap<String, SocketAddr> = HashMap::new();
    // Wait for clients and servers
    loop {
        // Read the first packet
        let (len, addr) = match socket.recv_from(&mut buffer) {
            Ok(s) => s,
            Err(err) => die(err),
        };
        // Ignore if this is IPv6
        if addr.is_ipv6() {
            log::warn!("Got packet from IPv6 address of {}", addr);
            continue;
        }
        // Parse the packet
        let packet = UDPMessage::parse(&buffer[..len]);
        if packet.is_none() {
            log::warn!("Got invalid packet from {}", addr);
            continue;
        }
        let packet = packet.unwrap();
        // Check the request
        match packet {
            UDPMessage::Server { service_name } => {
                // Check if same key exists in servers name
                if servers.contains_key(service_name) {
                    log::warn!("duplicate key {} from {}", service_name, addr);
                    // Format the packet
                    let response_length = UDPMessage::Error { message: "duplicate key" }.serialize(&mut buffer);
                    // Send it
                    let _ = socket.send_to(&buffer[..response_length], addr);
                    continue;
                }
            },
            UDPMessage::Client { service_name } => {
                
            },
            _ => {},
        };
    }
}