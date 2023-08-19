use postcard;
use std::{
    collections::HashMap,
    net::{SocketAddr, SocketAddrV4, UdpSocket},
    time::{Duration, Instant},
};

use crate::{
    messages::{PunchError, PunchMessage, UDPMessage},
    util::STUN_BUFFER_SIZE,
};

const SERVERS_CLEAN_UP_INTERVAL: Duration = Duration::from_secs(60 * 10);
const SLATE_SERVER: Duration = Duration::from_secs(60 * 5);

/// Spawn the STUN server which connects all clients and servers together
pub fn spawn_stun(listen: &str) -> ! {
    // Bind on address
    let socket = UdpSocket::bind(listen).expect("cannot bind UDP socket");
    log::info!("Listening on {}", socket.local_addr().unwrap());
    // Setup variables
    let mut buffer = [0; STUN_BUFFER_SIZE];
    let mut servers: HashMap<String, (SocketAddrV4, Instant)> = HashMap::new();
    let mut last_server_cleanup = Instant::now();
    // Wait for clients and servers
    loop {
        // Read the first packet
        let (len, addr) = socket
            .recv_from(&mut buffer)
            .expect("cannot receive datagrams");
        // Before doing stuff, clean up the hashmap if needed
        if last_server_cleanup.elapsed() > SERVERS_CLEAN_UP_INTERVAL {
            servers.retain(|_, (_, instead_date)| instead_date.elapsed() < SLATE_SERVER);
            last_server_cleanup = Instant::now();
        }
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
                servers.insert(service_name.to_owned(), (addr, Instant::now()));
                log::debug!("Added {} for {}", service_name, addr);
                // Send back the success message
                send_udp_packet(&UDPMessage::Ok, &socket, &addr);
            }
            UDPMessage::Client { service_name } => {
                // Check if the service name exists
                match servers.remove(service_name) {
                    Some((server_address, _)) => {
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

/// Sends an UDP packet from a socket to address
fn send_udp_packet(msg: &UDPMessage, socket: &std::net::UdpSocket, addr: &SocketAddrV4) {
    if let Ok(write_buffer) = postcard::to_vec::<&UDPMessage, STUN_BUFFER_SIZE>(&msg) {
        // Send it
        let _ = socket.send_to(&write_buffer, addr);
    }
}
