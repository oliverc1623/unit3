use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::io::prelude::*;
use std::fs::File;
use crate::geom::Pos3;

#[derive(Serialize, Deserialize)]
pub struct SaveFile {
    pub location: Location,
}

#[derive(Serialize, Deserialize)]
pub struct Location {
    pub x: f32,
    pub y:f32,
    pub z:f32,
}



pub fn parse_save(string_path: String) -> Result<SaveFile> {
    let mut save_string = String::new();

    let b = std::path::Path::new(&string_path).exists();
    let mut file:File;
    if !b {
            file = File::create(string_path).unwrap();
        }
        else{
            file = File::open(string_path).unwrap();
        }
        
        file.read_to_string(&mut save_string);
        let save: SaveFile = serde_json::from_str(save_string.as_mut_str())?;
        return Ok(save);
    
    
}

pub fn new_save(position: Pos3, string_path: String) -> Result<()>{
    let save_file = SaveFile {
        location:Location { 
            x:position.x,
            y:position.y,
            z:position.z
        } 
    };

    // Serialize it to a JSON string.
    let file = File::create(string_path);
    let j = serde_json::to_string(&save_file)?;

    
   if let mut buffer = file.unwrap() { 
        buffer.write_all(j.as_bytes());
   }
    // Print, write to a file, or send to an HTTP server.
    println!("{}", j);

    Ok(())
}



