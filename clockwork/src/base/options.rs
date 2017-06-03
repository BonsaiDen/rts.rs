// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Internal Dependencies ------------------------------------------------------
use ::{ConnectionID, Options};


/// Serializable wrapper type around `Options`.
#[derive(Debug, Serialize, Deserialize)]
pub struct RemoteOptions<O> {

    /// Unique ID of the client these options belong to.
    pub id: ConnectionID,

    /// Options of the client.
    pub data: O

}

impl<O> RemoteOptions<O> where O: Options {
    pub fn new(id: ConnectionID, options: O) -> Self {
        Self {
            id: id,
            data: options
        }
    }
}

