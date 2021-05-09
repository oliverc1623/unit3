use rodio::{SpatialSink, OutputStream, OutputStreamHandle};
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::BufReader;
use std::thread;
use std::time::Duration;
// const TESTSOUND: str = "content/music.ogg";

pub struct Sound {
    pub sink: SpatialSink,
}

impl Sound{
    pub fn add_sound<P: AsRef<Path>>(&self, path: P) {
        // println!("found file? {}", std::fs::File::open(&path).is_ok());
        let file = std::fs::File::open(path).unwrap();
        let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
        self.sink.append(source)
    }
    pub fn play_left(&self) {
        for i in 1..1001 {
            // thread::sleep(Duration::from_millis(5));
            self.sink.set_emitter_position([(i - 500) as f32 / 50.0, 0.0, 0.0]);
        }
    }
    pub fn playleft_to_right(&self) {
        for i in 1..1001 {
            self.sink.set_emitter_position([(i - 500) as f32 / 50.0, 0.0, 0.0]);
        }
        for i in 1..1001 {
            self.sink.set_emitter_position([-(i - 500) as f32 / 50.0, 0.0, 0.0]);
        }
        // self.sink.sleep_until_end();
    }
}

// println!("outputs: {}", rodio::OutputStream::try_default().is_ok());
// let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
// let sink = rodio::SpatialSink::try_new(
//     &handle,
//     [-10.0, 0.0, 0.0],
//     [1.0, 0.0, 0.0],
//     [-1.0, 0.0, 0.0],
// )
// .unwrap();

// // print!("Audio is being read: {}", std::fs::File::open("content/music.mp3").is_ok());
// let file = std::fs::File::open("content/music.ogg").unwrap();
// let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
// sink.append(source);   
