use postcard;
use std::{
    collections::HashMap,
    net::{SocketAddr, SocketAddrV4, UdpSocket},
};

use crate::{
    messages::{PunchError, PunchMessage, UDPMessage},
    util::{die, send_udp_packet, STUN_BUFFER_SIZE},
};

pub fn spawn_stun(listen: &str) -> ! {
    // Bind on address
    let socket = match UdpSocket::bind(listen) {
        Ok(s) => s,
        Err(err) => die(err),
    };
    // Setup variables
    let mut buffer = [0; STUN_BUFFER_SIZE];
    let mut servers: HashMap<String, SocketAddrV4> = HashMap::new();
    // Wait for clients and servers
    loop {
        // Read the first packet
        let (len, addr) = match socket.recv_from(&mut buffer) {
            Ok(s) => s,
            Err(err) => die(err),
        };
        // Ignore if this is IPv6
        let addr = match addr {
            SocketAddr::V4(v4) => v4,
            SocketAddr::V6(_) => {
                log::warn!("Got packet from IPv6 address of {}", addr);
                continue;
            }
        };
        // Parse the packet
        let packet = postcard::from_bytes::<UDPMessage<'_>>(&buffer[..len]);
        if let Err(err) = packet {
            log::warn!("Got invalid packet from {}: {}", addr, err);
            continue;
        }
        let packet = packet.unwrap();
        // Check the request
        match packet {
            UDPMessage::Server { service_name } => {
                // Check if same key exists in servers name
                if servers.contains_key(service_name) {
                    log::warn!("duplicate key {} from {}", service_name, addr);
                    // Send the packet
                    send_udp_packet(
                        &UDPMessage::Error {
                            0: PunchError::DuplicateKey,
                        },
                        &socket,
                        &addr,
                    );
                    continue;
                }
                // Add it to server list
                servers.insert(service_name.to_owned(), addr);
                // Send back the success message
                send_udp_packet(
                    &UDPMessage::Ok,
                    &socket,
                    &addr,
                );
            }
            UDPMessage::Client { service_name } => {}
            _ => {}
        };
    }
}
