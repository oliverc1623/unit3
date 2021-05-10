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
        self.sink.append(source);
    }
    pub fn play_left_to_right(&self, xdisp: f32) {
        for i in 1..(xdisp as i32 * 25) {
            thread::sleep(Duration::from_millis(2));
            println!("{}", (i - 500) as f32 / 50.0);
            self.sink.set_emitter_position([(i - 500) as f32 / 50.0, 0.0, 0.0]);
        }
    }
    pub fn play_bottom_to_top(&self, zdisp: f32){
        for i in 1..-(zdisp as i32 * 25) {
            thread::sleep(Duration::from_millis(2));
            println!("{}", (i - 500) as f32 / 50.0);
            self.sink.set_emitter_position([0.0, 0.0, (i - 500) as f32 / 50.0]);
        }
    }
    pub fn play_top_to_bottom(&self, zdisp: f32){
        for i in 1..(zdisp as i32 * 20) {
            thread::sleep(Duration::from_millis(2));
            println!("{}", -(i - 500) as f32 / 50.0);
            self.sink.set_emitter_position([0.0, 0.0, -(i - 500) as f32 / 50.0]);
        }
    }
    pub fn play_right_to_left(&self, xdisp: f32) {
        for i in 1..-(xdisp as i32 * 25) {
            thread::sleep(Duration::from_millis(2));
            println!("{}", -(i - 500) as f32 / 50.0);
            self.sink.set_emitter_position([-(i - 500) as f32 / 50.0, 0.0, 0.0]);
        }
    }
    // pub fn playleft_to_right(&self) {
    //     // self.sink.set_volume(10.0);
    //     for i in 1..1001 {
    //         // println!("left i: {} ", i);
    //         thread::sleep(Duration::from_millis(5));
    //         self.sink.set_emitter_position([(i - 500) as f32 / 50.0, 0.0, 0.0]);
    //     }
    //     for i in 1..1001 {
    //         // println!("right i: {} ", i);
    //         thread::sleep(Duration::from_millis(1));
    //         self.sink.set_emitter_position([-(i - 500) as f32 / 50.0, 0.0, 0.0]);
    //     }
    //     // println!("done playing; sink is empty? {} ", self.sink.empty());
    //     // std::thread::sleep(std::time::Duration::from_secs(1));
    // }
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
