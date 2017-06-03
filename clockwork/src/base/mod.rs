// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// Internal -------------------------------------------------------------------
use ::Config;


// Modules --------------------------------------------------------------------
mod client;
mod connection;
mod input;
mod options;
mod server;


// Re-Exports -----------------------------------------------------------------
pub use self::client::{ClockworkClient, Migration};
pub use self::connection::RemoteConnection;
pub use self::input::RemoteInput;
pub use self::options::RemoteOptions;
pub use self::server::ClockworkServer;


/// Unique ID for connection identification.
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, Ord, PartialOrd, Serialize, Deserialize)]
pub struct ConnectionID {
    id: u32
}

impl ConnectionID {

    fn new(id: u32) -> Self {
        Self {
            id: id
        }
    }

}

impl PartialEq<HostID> for ConnectionID {
    fn eq(&self, other: &HostID) -> bool {
        self.id == other.id
    }
}

impl From<HostID> for ConnectionID {
    fn from(host_id: HostID) -> ConnectionID {
        ConnectionID{
            id: host_id.id
        }
    }
}


/// A variant of a `ConnectionID` which references the remote that hosts the game.
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, Ord, PartialOrd, Serialize, Deserialize)]
pub struct HostID {
    id: u32
}

impl HostID {
    fn new(id: ConnectionID) -> Self {
        Self {
            id: id.id
        }
    }
}

impl PartialEq<ConnectionID> for HostID {
    fn eq(&self, other: &ConnectionID) -> bool {
        self.id == other.id
    }
}


// Internal Helpers -----------------------------------------------------------
pub fn connection_id_from_packet(config: &Config, packet: &[u8]) -> Option<ConnectionID> {
    if packet.len() >= 8 && packet[0..4] == config.protocol_header {
        Some(ConnectionID::new(
            (packet[4] as u32) << 24 | (packet[5] as u32) << 16 |
            (packet[6] as u32) << 8  |  packet[7] as u32
        ))

    } else {
        None
    }
}

pub fn base_packet(id: ConnectionID, config: &Config, tick: u8) -> Vec<u8> {
    let mut packet = Vec::new();
    packet.extend_from_slice(&config.protocol_header[..]);
    packet.push((id.id >> 24) as u8);
    packet.push((id.id >> 16) as u8);
    packet.push((id.id >> 8) as u8);
    packet.push(id.id as u8);
    packet.push(tick);
    packet
}

