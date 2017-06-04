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


// Structs --------------------------------------------------------------------
pub struct Unit {
    sprite: Sprite,
    selected: bool
}

impl Unit {

    pub fn new(x: f32, y: f32) -> Self {

        let mut sprite = Sprite::new();
        sprite.set_tile_size(2, 2);
        sprite.set_size(64.0, 64.0);
        sprite.set_position(x, y);
        sprite.set_tile(8);

        Self {
            sprite: sprite,
            selected: false
        }

    }

    pub fn set_selected(&mut self, active: bool) {
        self.selected = active;
    }

    pub fn is_hit(&self, x: f32, y: f32) -> bool {
        self.sprite.hit(x, y)
    }

    pub fn draw(&self, view: &mut SpriteView) {
        if self.selected {
            let (x, y) = self.sprite.position();
            let mut selection = Sprite::new();
            selection.set_size(68.0, 68.0);
            selection.set_position(x - 2.0, y - 2.0);
            view.draw_sprite(&selection);
        }
        view.draw_sprite(&self.sprite);
    }

}


// Example --------------------------------------------------------------------
struct Demo {
    view: SpriteView,
    units: Vec<Unit>,
    cursor: Sprite,
    scroll: (i32, i32)
}

impl Demo {

    fn new(mut view: SpriteView) -> Self {

        let mut cursor = Sprite::new();
        cursor.set_size(32.0, 32.0);

        Self {
            view: view,
            units: Vec::new(),
            cursor: cursor,
            scroll: (0, 0)
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
            let (x, y) = ((x + self.scroll.0) as f32, (y + self.scroll.1) as f32);

            let mut any = false;
            for unit in &mut self.units {
                unit.set_selected(false);
                if unit.is_hit(x, y) && !any {
                    unit.set_selected(true);
                    any = true;
                }
            }

            if !any {
                self.units.push(Unit::new(x as f32, y as f32));
            }

        }

        if mouse.was_pressed(Button::Right) {

            let (x, y) = mouse.get(Button::Right).position();
            let (x, y) = ((x + self.scroll.0) as f32, (y + self.scroll.1) as f32);

            let mut any = false;
            self.units.retain(|unit| {
                if unit.is_hit(x, y) && !any {
                    any = true;
                    false

                } else {
                    true
                }
            });

        }

        let (px, py) = mouse.position();
        self.cursor.set_position((px + self.scroll.0) as f32, (py + self.scroll.1) as f32);

        for unit in &self.units {
            unit.draw(&mut self.view);
        }

        self.view.draw_sprite(&self.cursor);

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

