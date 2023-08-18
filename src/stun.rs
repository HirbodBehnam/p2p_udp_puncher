use postcard;
use std::{
    collections::HashMap,
    net::{SocketAddr, SocketAddrV4, UdpSocket},
};

use crate::{
    messages::{PunchError, PunchMessage, UDPMessage},
    util::{send_udp_packet, STUN_BUFFER_SIZE},
};

/// Spawn the STUN server which connects all clients and servers together
pub fn spawn_stun(listen: &str) -> ! {
    // Bind on address
    let socket = UdpSocket::bind(listen).expect("cannot bind UDP socket");
    log::info!("Listening on {}", socket.local_addr().unwrap());
    // Setup variables
    let mut buffer = [0; STUN_BUFFER_SIZE];
    let mut servers: HashMap<String, SocketAddrV4> = HashMap::new();
    // Wait for clients and servers
    loop {
        // Read the first packet
        let (len, addr) = socket.recv_from(&mut buffer).expect("cannot receive datagrams");
        // Ignore if this is IPv6
        let addr = match addr {
            SocketAddr::V4(v4) => v4,
            SocketAddr::V6(_) => {
                log::warn!("Got packet from IPv6 address of {}", addr);
                continue;
            }
        };
        // Parse the packet
        let packet = match postcard::from_bytes::<UDPMessage<'_>>(&buffer[..len]) {
            Err(err) => {
                log::warn!("Got invalid packet from {}: {}", addr, err);
                continue;
            }
            Ok(pkt) => pkt,
        };
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
                log::debug!("Added {} for {}", service_name, addr);
                // Send back the success message
                send_udp_packet(&UDPMessage::Ok, &socket, &addr);
            }
            UDPMessage::Client { service_name } => {
                // Check if the service name exists
                match servers.remove(service_name) {
                    Some(server_address) => {
                        log::debug!(
                            "Matching client {} with server {} via key {}",
                            addr,
                            server_address,
                            service_name
                        );
                        // Send message to server
                        send_udp_packet(
                            &UDPMessage::Punch {
                                0: PunchMessage::STUN { 0: addr },
                            },
                            &socket,
                            &server_address,
                        );
                        // Send message to client
                        send_udp_packet(
                            &UDPMessage::Punch {
                                0: PunchMessage::STUN { 0: server_address },
                            },
                            &socket,
                            &addr,
                        );
                    }
                    // No server was found!
                    None => {
                        log::warn!(
                            "{} requested for {} which does not exist",
                            addr,
                            service_name
                        );
                        send_udp_packet(
                            &UDPMessage::Error {
                                0: PunchError::NoServer,
                            },
                            &socket,
                            &addr,
                        );
                    }
                };
            }
            _ => {}
        };
    }
}
