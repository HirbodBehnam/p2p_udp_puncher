use std::{net::SocketAddrV4, str};

use serde::{Deserialize, Serialize};

/// All possible messages which can be sent from or to all apps
#[derive(Serialize, Deserialize, Debug)]
pub enum UDPMessage<'a> {
    /// Client wants to connect to TURN server
    Client {
        service_name: &'a str,
    },
    /// Server advertising itself to TURN server
    Server {
        service_name: &'a str,
    },
    // An error...
    Error(PunchError),
    // Punch packet
    Punch(PunchMessage),
    /// No true value, just ignore this packet
    KeepAlive,
    /// Something was ok. Client knows what it is
    Ok,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum PunchError {
    /// There is another server with this key
    DuplicateKey,
    /// No server is listening with this key
    NoServer,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum PunchMessage {
    PeerHandshake1,
    PeerHandshake2,
    PeerHandshake3,
    TURN(SocketAddrV4),
}
