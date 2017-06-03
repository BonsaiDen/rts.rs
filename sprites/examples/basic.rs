// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// Crates ---------------------------------------------------------------------
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
    sprite: Option<Sprite>,
    scroll: (i32, i32)
}

impl Demo {
    fn new(mut view: SpriteView) -> Self {
        Self {
            view: view,
            sprite: None,
            scroll: (0, 0)
        }
    }

    fn create_sprite(&mut self) {
        let mut sprite = self.view.create_sprite().unwrap();
        sprite.set_size(32.0, 32.0);
        sprite.set_position(0.0, 0.0);
        sprite.set_tile(22);

        self.view.update_sprite(&sprite);
        self.sprite = Some(sprite);
    }

    fn destroy_sprite(&mut self) {
        if let Some(sprite) = self.sprite.take() {
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

            if self.sprite.is_none() {
                self.create_sprite();
            }

            if let Some(ref mut sprite) = self.sprite {
                let (x, y) = mouse.get(Button::Left).position();
                sprite.set_position((x + self.scroll.0) as f32, (y + self.scroll.1) as f32);
                self.view.update_sprite(&sprite);
            }

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


