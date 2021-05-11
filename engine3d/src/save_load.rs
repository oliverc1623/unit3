use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::io::prelude::*;
use std::fs::File;
use crate::geom::Pos3;

#[derive(Serialize, Deserialize)]
pub struct SaveFile {
    pub player_location: Location,
    pub button_location: Location,
    pub wall_location: Location
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

pub fn new_save(player_position: Pos3, button_position: Pos3, wall_position: Pos3, string_path: String) -> Result<()>{
    let save_file = SaveFile {
        player_location:Location { 
            x:player_position.x,
            y:player_position.y,
            z:player_position.z
        },
        button_location:Location { 
            x:button_position.x,
            y:button_position.y,
            z:button_position.z
        },
        wall_location:Location { 
            x:wall_position.x,
            y:wall_position.y,
            z:wall_position.z
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



