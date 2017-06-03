// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// STD Dependencies -----------------------------------------------------------
use std::hash::Hash;
use std::fmt::Debug;
use std::net::SocketAddr;


// External Dependencies ------------------------------------------------------
use serde::Serialize;
use serde::de::DeserializeOwned;


// Internal Dependencies ------------------------------------------------------
use ::{ConnectionID, HostID};


/// A trait for the implementation of the initial game options.
pub trait Options: Send + Hash + Debug + Serialize + DeserializeOwned {}

/// A trait for the implementation of the game inputs.
pub trait Input: Send + Copy + Clone + Debug + Serialize + DeserializeOwned {}

/// A trait for the implementation of the overall game state.
pub trait State<O, I, R>: Default {
    fn is_ready(&self) -> bool;
    fn init(&mut self, HostID, &[(ConnectionID, SocketAddr)], &mut R);
    fn tick(&mut self, HostID, &[(ConnectionID, SocketAddr)]);
    fn apply_options(&mut self, HostID, &[(ConnectionID, O)]);
    fn apply_input(&mut self, HostID, ConnectionID, I);
}

