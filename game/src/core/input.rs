// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// External Dependencies ------------------------------------------------------
use clockwork::Input;


// Local Game Input -----------------------------------------------------------
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum GameInput {
    // TODO how to abstract unit commands etc?
    LeftClick(u8, u8),
    Idle
}

impl Input for GameInput {}

impl Default for GameInput {
    fn default() -> Self {
       GameInput::Idle
    }
}


