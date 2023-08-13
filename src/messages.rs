use std::{cmp, str};

/// All possible messages which can be sent from or to all apps
pub enum UDPMessage<'a> {
    Client {
        service_name: &'a str,
    },
    Server {
        service_name: &'a str,
    },
    Error {
        message: &'a str,
    },
    Punch,
    /// No true value, just ignore this packet
    KeepAlive,
}

impl UDPMessage<'_> {
    /// Parse a UDP message from a read buffer.
    /// Parsing is done with zero-copy principle
    pub fn parse<'a>(buf: &'a [u8]) -> Option<UDPMessage<'a>> {
        // Nothing should be zero sized
        if buf.len() == 0 {
            panic!("zero sized buffer");
        }
        // All of the messages has a non-existent string or a valid utf8 string
        let message = str::from_utf8(&buf[1..]).unwrap();
        // Check type of message
        match buf[0] {
            0 => Some(UDPMessage::Client { service_name: message }),
            1 => Some(UDPMessage::Server { service_name: message }),
            2 => Some(UDPMessage::Error { message }),
            3 => Some(UDPMessage::Punch),
            4 => Some(UDPMessage::KeepAlive),
            _ => None,
        }
    }

    /// 
    pub fn serialize(&self, buf: &mut [u8]) -> usize {
        // Check empty buffer
        if buf.len() == 0 {
            panic!("zero sized buffer");
        }
        // Write the message type
        match self {
            UDPMessage::Client {..} => buf[0] = 0,
            UDPMessage::Server {..} => buf[0] = 1,
            UDPMessage::Error {..} => buf[0] = 2,
            UDPMessage::Punch => buf[0] = 3,
            UDPMessage::KeepAlive => buf[0] = 4,
        };
        // Copy payload (if needed)
        match self {
            UDPMessage::Client { service_name } => 1 + Self::copy_slice(&mut buf[1..], service_name.as_bytes()),
            UDPMessage::Server { service_name } => 1 + Self::copy_slice(&mut buf[1..], service_name.as_bytes()),
            UDPMessage::Error { message } => 1 + Self::copy_slice(&mut buf[1..], message.as_bytes()),
            UDPMessage::Punch => 1,
            UDPMessage::KeepAlive => 1,
        }
    }
    
    /// Copy one slice to another and return the number of bytes copied
    fn copy_slice(dest: &mut [u8], src: &[u8]) -> usize {
        let to_copy = cmp::min(dest.len(), src.len());
        dest[..to_copy].copy_from_slice(&src[..to_copy]);
        return to_copy
    }
}