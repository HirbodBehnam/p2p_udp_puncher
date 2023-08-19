use std::{fmt, net::SocketAddrV4, time::Duration};

/// Size of buffer of network sockets for connecting to STUN server
pub const STUN_BUFFER_SIZE: usize = 128;

/// Local address on which we should bind in order to open the UDP socket as client
pub const LOCAL_UDP_BIND_ADDRESS: SocketAddrV4 =
    SocketAddrV4::new(std::net::Ipv4Addr::new(0, 0, 0, 0), 0);

/// The buffer size which is used to copy two UDP sockets
pub const FORWARD_BUFFER_SIZE: usize = 4 * 1024;

/// How long to wait before a socket times out
pub const SOCKET_TIMEOUT: Duration = Duration::from_secs(5);

/// Kill the program with an error message
pub fn die<E: fmt::Debug>(error: E) -> ! {
    log::error!("{:?}", error);
    std::process::exit(1);
}
