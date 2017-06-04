// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// Crates ---------------------------------------------------------------------
extern crate rodio;


// STD Dependencies -----------------------------------------------------------
use std::thread;
use std::fs::File;
use std::path::PathBuf;
use std::io::BufReader;
use std::sync::mpsc::{channel, Sender};


// External Dependencies ------------------------------------------------------
use rodio::Source;


// Public Interface -----------------------------------------------------------
enum AudioCommand {
    EffectPlay(PathBuf, f32),
    MusicStart(PathBuf),
    MusicStop
}

pub struct AudioQueue {
    sender: Sender<Option<AudioCommand>>,
    thread: Option<thread::JoinHandle<()>>
}

impl AudioQueue {

    pub fn new() -> Self {

        let (sender, receiver) = channel::<Option<AudioCommand>>();
        let handle = thread::spawn(move || {

            let endpoint = rodio::get_default_endpoint().unwrap();
            while let Ok(Some(command)) = receiver.recv() {
                match command {
                    AudioCommand::EffectPlay(path, speed) => {
                        println!("[Audio] Play {:?}", path);
                        let file = File::open(path).unwrap();
                        let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
                        rodio::play_raw(&endpoint, source.convert_samples().speed(speed));
                    },
                    _ => {}
                }
            }

        });

        Self {
            sender: sender,
            thread: Some(handle)
        }

    }

    pub fn play_effect(&mut self, path: PathBuf, speed: Option<f32>) {
        self.sender.send(Some(AudioCommand::EffectPlay(path, speed.unwrap_or(1.0)))).ok();
    }

    pub fn stop(&mut self) {
        if let Some(handle) = self.thread.take() {
            self.sender.send(None).ok();
            handle.join().ok();
        }
    }

}

impl Drop for AudioQueue {
    fn drop(&mut self) {
        self.stop()
    }
}

