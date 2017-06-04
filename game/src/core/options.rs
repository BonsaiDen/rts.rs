// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// External Dependencies ------------------------------------------------------
use clockwork::Options;


// Local Game Options ---------------------------------------------------------
#[derive(Debug, Hash, Clone, Serialize, Deserialize)]
pub struct GameOptions {
    pub min_players: u8,
    pub random_seed: [u8; 4]
}

impl Options for GameOptions {}

impl Default for GameOptions {
    fn default() -> Self {
        Self {
            min_players: 3,
            random_seed: [0, 0, 0, 0]
        }
    }
}

