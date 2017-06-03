// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// STD Dependencies -----------------------------------------------------------
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};


// Internal Dependencies ------------------------------------------------------
use base::ConnectionID;


/// Remote connection information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteConnection {

    /// The unique client id.
    id: ConnectionID,

    /// Address of the client.
    address: RemoteAddr,

    /// Order of the client when performing host migration.
    order: u8

}

impl RemoteConnection {

    pub fn new(id: ConnectionID, address: SocketAddr, order: u8) -> Self {
        Self {
            id: id,
            address: address.into(),
            order: order
        }
    }

    pub fn id(&self) -> ConnectionID {
        self.id
    }

    pub fn address(&self) -> SocketAddr {
        self.address.into()
    }

    pub fn set_address(&mut self, address: SocketAddr) {
        self.address = address.into();
    }

    pub fn order(&self) -> u8 {
        self.order
    }

    pub fn set_order(&mut self, order: u8) {
        self.order = order;
    }

}


// Helpers --------------------------------------------------------------------
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
struct RemoteAddr {
    ip: [u8; 4],
    port: u16
}

impl From<SocketAddr> for RemoteAddr {
    // TODO support IPv6 with a prefix byte
    fn from(addr: SocketAddr) -> RemoteAddr {
        match addr.ip() {
            IpAddr::V4(ip) => RemoteAddr {
                ip: ip.octets(),
                port: addr.port()
            },
            IpAddr::V6(..) => panic!("IPV6 is currently not supported")
        }
    }
}

impl Into<SocketAddr> for RemoteAddr {
    // TODO support IPv6 with a prefix byte
    fn into(self) -> SocketAddr {
        SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::new(self.ip[0], self.ip[1], self.ip[2], self.ip[3]),
            self.port
        ))
    }
}

