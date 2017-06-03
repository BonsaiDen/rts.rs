// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// STD Dependencies -----------------------------------------------------------
use std::net;
use std::fmt;
use std::iter;
use std::io::Error;
use std::sync::mpsc::TryRecvError;


/// Non-blocking abstraction over a UDP socket.
pub struct Socket {
    socket: net::UdpSocket,
    buffer: Vec<u8>
}

impl Socket {

    /// Tries to create a new UDP socket by binding to the specified address.
    pub fn new<T: net::ToSocketAddrs>(
        address: T,
        max_packet_size: usize

    ) -> Result<Self, Error> {

        let socket = net::UdpSocket::bind(address)?;
        socket.set_nonblocking(true)?;

        Ok(Socket {
            buffer: iter::repeat(0).take(max_packet_size).collect(),
            socket: socket
        })

    }

    /// Attempts to return a incoming packet on this socket without blocking.
    pub fn try_recv(&mut self) -> Result<(net::SocketAddr, Vec<u8>), TryRecvError> {
        if let Ok((len, src)) = self.socket.recv_from(&mut self.buffer) {
            Ok((src, self.buffer[..len].to_vec()))

        } else {
            Err(TryRecvError::Empty)
        }
    }

    /// Send data on the socket to the given address. On success, returns the
    /// number of bytes written.
    pub fn send_to(
        &mut self,
        data: &[u8],
        addr: net::SocketAddr

    ) -> Result<usize, Error> {
        self.socket.send_to(data, addr)
    }

}

impl fmt::Debug for Socket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Socket({:?})", self.socket)
    }
}

