// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// STD Dependencies -----------------------------------------------------------
use std::time::Duration;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};


/// Client and server configuration.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Config {

    /// Number of packets send per second during game setup or pause,
    /// defaulting to `10`.
    ///
    /// > Note: Due to network round trip times the actual game tick rate
    /// > will always be half of this value.
    pub low_tick_rate: u64,

    /// Number of packets send per second during game synchronization,
    /// defaulting to `30`.
    ///
    /// > Note: Due to network round trip times the actual game tick rate
    /// > will always be half of this value.
    pub high_tick_rate: u64,

    /// Maximum bytes that can be received or send in one packet, defaulting to
    /// `1400`.
    pub packet_max_size: usize,

    /// 32-Bit Protocol ID used to identify game packets, defaulting to
    /// `[1, 2, 3, 4]`.
    pub protocol_header: [u8; 4],

    /// Local address for a server to bind to for receiving data, defaulting to
    /// `0.0.0.0:7156`.
    pub server_addr: SocketAddr,

    /// Maximum number of clients allowed to connect to a server, defaulting to
    /// `16`.
    pub server_max_clients: u8,

    /// Remote address for a client to connect to, defaulting to
    /// `127.0.0.1:7156`.
    pub remote_addr: SocketAddr,

    /// Maximum time in milliseconds between any two packets before a
    /// remote is considered pending will force a game pause, defaulting to
    /// `2500`.
    pub remote_pending_threshold: Duration,

    /// Maximum time in milliseconds between any two packets before a
    /// remote is fully timed out and disconnected from a server, defaulting
    /// to `5000`.
    pub remote_timeout_threshold: Duration

}

impl Config {

    /// Return the local server address for use in host migration.
    pub fn local_server_addr(&self) -> SocketAddr {
        SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::new(127, 0, 0, 1),
            self.server_addr.port()
        ))
    }

    /// Returns the remote server address for use in host migration.
    pub fn remote_host_addr(&self, mut address: SocketAddr) -> SocketAddr {
        address.set_port(self.server_addr.port());
        address
    }

}

impl Default for Config {

    fn default() -> Self {
        Config {
            high_tick_rate: 30,
            low_tick_rate: 10,
            protocol_header: [1, 2, 3, 4],
            packet_max_size: 1400,
            server_addr: SocketAddr::V4(
                SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 7156)
            ),
            server_max_clients: 16,
            remote_addr: SocketAddr::V4(
                SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 7156)
            ),
            remote_pending_threshold: Duration::from_millis(500),
            remote_timeout_threshold: Duration::from_millis(1000)
        }
    }

}

