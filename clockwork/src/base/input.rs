// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Internal Dependencies ------------------------------------------------------
use ::{ConnectionID, Input};


/// Serializable wrapper type around `Input`, representing the complete remote
/// input.
#[derive(Debug, Serialize, Deserialize)]
pub struct RemoteInput<I> {

    /// Unique ID of the client this input belongs to.
    pub id: ConnectionID,

    /// Input state of the client.
    pub data: Vec<I>,

    /// Sequence number of the input.
    pub sequence: u8

}

impl<I> RemoteInput<I> where I: Input {
    pub fn new(id: ConnectionID, sequence: u8, input: Vec<I>) -> Self {
        Self {
            id: id,
            sequence: sequence,
            data: input
        }
    }
}

