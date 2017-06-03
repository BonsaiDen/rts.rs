// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// External Dependencies ------------------------------------------------------
use gfx;
use gfx_device_gl;
use renderer::{Key, Keyboard, Button, Mouse, Renderable, RenderTarget};
use clockwork::{Clockwork, Event};


// Internal Dependencies ------------------------------------------------------
pub use core::{GameInput, GameOptions, GameState};


// Game Implementation --------------------------------------------------------
pub struct Game {
    client: Clockwork<GameState, GameOptions, GameInput, RenderTarget>,
    options: GameOptions,
    scroll: (i32, i32),
    target: RenderTarget
}

impl Game {
    pub fn new(
        client: Clockwork<GameState, GameOptions, GameInput, RenderTarget>,
        options: GameOptions,
        target: RenderTarget

    ) -> Game {
        Self {
            client: client,
            options: options,
            scroll: (0, 0),
            target: target
        }
    }
}

impl Renderable for Game {

    fn tick(&mut self) where Self: Sized {

        while let Ok(event) = self.client.try_recv(&mut self.target) {
            match event {
                Event::HostConnect(address, host_id, local_id) => {
                    println!("[Network] Connected to host {:?}({:?}) as {:?}, but not yet ready...", address, host_id, local_id);
                    self.client.set_options(self.options.clone());
                },
                Event::HostReady(address, host_id, local_id) => {
                    println!("[Network] Connected to host {:?}({:?}) as {:?}, now ready!", address, host_id, local_id)
                },
                Event::HostReconnect(address, host_id, local_id) => {
                    println!("[Network] Reconnecting to host {:?}({:?}) as {:?}...", address, host_id, local_id);
                },
                Event::RemoteJoined(address, id) => {
                    println!("[Network] Remote {:?}{:?} joined", address, id);
                },
                Event::RemoteOptions => {
                    println!("[Network] Options have changed");
                },
                Event::RemoteLeft(address, id) => {
                    println!("[Network] Remote {:?}{:?} left", address, id);
                },
                Event::Error(err) => {
                    println!("[Network] Error: {:?}", err);
                    self.client.disconnect();
                    return;
                }
            }
        }

        self.client.send();

    }

    fn draw(&mut self, mut encoder: &mut gfx::Encoder<
        gfx_device_gl::Resources,
        gfx_device_gl::CommandBuffer

    >, keyboard: &Keyboard, mouse: &Mouse) where Self: Sized {

        // Scrolling
        if keyboard.is_pressed(Key::A) {
            self.scroll.0 -= 12;
        }

        if keyboard.is_pressed(Key::D) {
            self.scroll.0 += 12;
        }

        if keyboard.is_pressed(Key::W) {
            self.scroll.1 -= 12;
        }

        if keyboard.is_pressed(Key::S) {
            self.scroll.1 += 12;
        }

        // Map
        let input = if let Some(ref mut tile_grid) = self.client.state().tile_grid {

            self.scroll = tile_grid.scroll_to(self.scroll.0, self.scroll.1);

            tile_grid.draw(&mut encoder);

            // Input
            if mouse.was_pressed(Button::Left) {
                let (x, y) = mouse.get(Button::Left).position();
                let p = tile_grid.screen_to_grid(x, y);
                Some(GameInput::LeftClick(p.0 as u8, p.1 as u8))

            } else {
                None
            }

        } else {
            None
        };

        if let Some(input) = input {
            self.client.queue_input(input);
        }

    }

}

