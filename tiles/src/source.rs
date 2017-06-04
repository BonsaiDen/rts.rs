// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// Traits ---------------------------------------------------------------------
pub trait TileSource: Default {
    fn set_tile_index(&mut self, x: i32, y: i32, index: u32) -> bool where Self: Sized;
    fn get_tile_index(&self, x: i32, y: i32) -> Option<u32> where Self: Sized;
    fn width(&self) -> u32 where Self: Sized;
    fn height(&self) -> u32 where Self: Sized;
    fn index(&self, index: usize) -> u32 where Self: Sized;
    fn indices(&self) -> &[u32] where Self: Sized;
}

