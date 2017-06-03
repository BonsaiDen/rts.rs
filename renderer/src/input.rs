// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// STD Dependencies -----------------------------------------------------------
use std::iter;
use std::marker::PhantomData;

// External Dependencies ------------------------------------------------------
use glutin::{MouseButton, VirtualKeyCode};


// Traits ---------------------------------------------------------------------
pub trait AdvanceableState {
    fn advance(&self) -> Self where Self: Sized;
    fn reset(&self) -> Self where Self: Sized;
    fn was_pressed(&self) -> bool where Self: Sized;
    fn was_released(&self) -> bool where Self: Sized;
    fn is_pressed(&self) -> bool where Self: Sized;
    fn is_released(&self) -> bool where Self: Sized;
}

pub trait WithPosition {
}


// Keyboad --------------------------------------------------------------------
#[derive(Debug, PartialEq, Eq)]
pub enum Key {
    W = 0,
    A = 1,
    S = 2,
    D = 3,
    Unknown = 4
}

impl From<VirtualKeyCode> for Key {
    fn from(code: VirtualKeyCode) -> Self {
        match code {
            VirtualKeyCode::W => Key::W,
            VirtualKeyCode::A => Key::A,
            VirtualKeyCode::S => Key::S,
            VirtualKeyCode::D => Key::D,
            _ => Key::Unknown
        }
    }
}

impl Into<usize> for Key {
    fn into(self) -> usize {
        self as usize
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum KeyState {
    WasPressed,
    Pressed,
    WasReleased,
    Released
}

impl AdvanceableState for KeyState {

    fn advance(&self) -> Self {
        match *self {
            KeyState::WasReleased => KeyState::Released,
            KeyState::WasPressed => KeyState::Pressed,
            _ => *self
        }
    }

    fn reset(&self) -> Self {
        match *self {
            KeyState::Pressed => KeyState::WasReleased,
            _ => *self
        }
    }

    fn was_pressed(&self) -> bool where Self: Sized {
        *self == KeyState::WasPressed
    }

    fn was_released(&self) -> bool where Self: Sized {
        *self == KeyState::WasReleased
    }

    fn is_pressed(&self) -> bool where Self: Sized {
        *self == KeyState::Pressed || *self == KeyState::WasPressed
    }

    fn is_released(&self) -> bool where Self: Sized {
        *self == KeyState::Released || *self == KeyState::WasReleased
    }

}

impl Default for KeyState {
    fn default() -> Self {
        KeyState::Released
    }
}


// Mouse ----------------------------------------------------------------------
#[derive(Debug, PartialEq, Eq)]
pub enum Button {
    Left = 0,
    Right = 1,
    Unknown = 2
}

impl From<MouseButton> for Button {
    fn from(code: MouseButton) -> Self {
        match code {
            MouseButton::Left => Button::Left,
            MouseButton::Right => Button::Right,
            _ => Button::Unknown
        }
    }
}

impl Into<usize> for Button {
    fn into(self) -> usize {
        self as usize
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ButtonState {
    WasPressed(i32, i32),
    Pressed(i32, i32),
    WasReleased(i32, i32),
    Released(i32, i32)
}

impl ButtonState {

    pub fn position(&self) -> (i32, i32) {
        match *self {
            ButtonState::WasPressed(x, y)
            | ButtonState::Pressed(x, y)
            | ButtonState::WasReleased(x, y)
            | ButtonState::Released(x, y) => (x, y)
        }
    }

}

impl AdvanceableState for ButtonState {

    fn advance(&self) -> Self {
        match *self {
            ButtonState::WasReleased(x, y) => ButtonState::Released(x, y),
            ButtonState::WasPressed(x, y) => ButtonState::Pressed(x, y),
            _ => *self
        }
    }

    fn reset(&self) -> Self {
        match *self {
            ButtonState::Pressed(x, y) => ButtonState::WasReleased(x, y),
            _ => *self
        }
    }

    fn was_pressed(&self) -> bool where Self: Sized {
        if let ButtonState::WasPressed(_, _) = *self {
            true

        } else {
            false
        }
    }

    fn was_released(&self) -> bool where Self: Sized {
        if let ButtonState::WasReleased(_, _) = *self {
            true

        } else {
            false
        }
    }

    fn is_pressed(&self) -> bool where Self: Sized {
        if let ButtonState::Pressed(_, _) = *self {
            true

        } else {
            self.was_pressed()
        }
    }

    fn is_released(&self) -> bool where Self: Sized {
        if let ButtonState::Released(_, _) = *self {
            true

        } else {
            self.was_released()
        }
    }

}

impl Default for ButtonState {
    fn default() -> Self {
        ButtonState::Released(-1, -1)
    }
}


// Input ----------------------------------------------------------------------
pub struct InputState<I, T> {
    index: PhantomData<I>,
    fields: Vec<T>
}

impl<I, T> InputState<I, T> where T: Default + Clone + AdvanceableState, I: Into<usize> {

    pub fn new(size: usize) -> Self {
        Self {
            fields: iter::repeat(T::default()).take(size).collect(),
            index: PhantomData
        }
    }

    pub fn was_pressed(&self, index: I) -> bool {
        self.fields[index.into()].was_pressed()
    }

    pub fn is_pressed(&self, index: I) -> bool {
        self.fields[index.into()].is_pressed()
    }

    pub fn was_released(&self, index: I) -> bool {
        self.fields[index.into()].was_released()
    }

    pub fn is_released(&self, index: I) -> bool {
        self.fields[index.into()].is_released()
    }

    pub fn set(&mut self, index: I, to: T) {
        self.fields[index.into()] = to;
    }

    pub fn advance(&mut self) {
        for value in &mut self.fields {
            *value = value.advance();
        }
    }

    pub fn reset(&mut self) {
        for value in &mut self.fields {
            *value = value.reset();
        }
    }

    pub fn get(&self, index: I) -> &T {
        &self.fields[index.into()]
    }

}

pub type Keyboard = InputState<Key, KeyState>;
pub type Mouse = InputState<Button, ButtonState>;

