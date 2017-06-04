// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// Crates ---------------------------------------------------------------------
extern crate rand;
extern crate sprites;
extern crate renderer;


// STD Dependencies -----------------------------------------------------------
use std::path::Path;


// External Dependencies ------------------------------------------------------
use sprites::{SpriteSheet, SpriteView, Sprite};
use renderer::{Key, Keyboard, Button, Mouse, Renderable, Encoder};


// Example --------------------------------------------------------------------
struct Demo {
    view: SpriteView,
    sprites: Vec<Sprite>,
    scroll: (i32, i32)
}

impl Demo {

    fn new(view: SpriteView) -> Self {
        Self {
            view: view,
            sprites: Vec::new(),
            scroll: (0, 0)
        }
    }

    fn create_sprite(&mut self, x: i32, y: i32) {
        let mut sprite = self.view.create_sprite().unwrap();
        sprite.set_size(32.0, 32.0);
        sprite.set_position(x as f32, y as f32);

        let tile: u8 = rand::random();
        sprite.set_tile(tile as u32);

        self.view.update_sprite(&sprite);
        self.sprites.push(sprite)
    }

    fn destroy_sprite(&mut self) {
        if let Some(sprite) = self.sprites.pop() {
            self.view.destroy_sprite(sprite);
        }
    }

}

impl Renderable for Demo {

    fn tick(&mut self) where Self: Sized {

    }

    fn draw(&mut self, mut encoder: &mut Encoder, keyboard: &Keyboard, mouse: &Mouse) where Self: Sized {

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

        if mouse.was_pressed(Button::Left) {
            let (x, y) = mouse.get(Button::Left).position();
            let (x, y) = (x + self.scroll.0, y + self.scroll.1 as i32);
            self.create_sprite(x, y);
        }

        if mouse.was_pressed(Button::Right) {
            self.destroy_sprite();
        }

        self.view.scroll_to(self.scroll.0, self.scroll.1);
        self.view.draw(&mut encoder);

    }

}

// Demo -----------------------------------------------------------------------
fn main() {
    renderer::run::<Demo, _>("Sprites", 640, 480, 60, 10, |mut target| {

        let sheet = SpriteSheet::new(&mut target.factory, &Path::new("../assets/textures/tileset.png"), 16).unwrap();
        let view = SpriteView::new(
            &mut target.factory,
            target.color.clone(),
            target.width,
            target.height,
            sheet,
            128
        );

        Demo::new(view)

    });
}


