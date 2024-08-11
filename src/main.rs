use std::io::Read;

use log::error;
pub mod constants;
pub mod shapes;
pub mod types;
use types::Room;

fn main() {
    let my_path = std::path::Path::new("./bins");
    if !my_path.exists() {
        println!("Folder not found");
        println!("{}", std::path::absolute(my_path).unwrap().display());
        return;
    }

    // get all files in the folder
    let entries = std::fs::read_dir(my_path).unwrap();
    for entry in entries {
        // error handling
        if entry.is_err() {
            error!("Error getting file: {:?}", entry.err());
            continue;
        }
        let entry = entry.unwrap();
        let path = entry.path();

        // is file a .room file?
        if !path.is_file() || !path.exists() || path.extension().unwrap() != "room" {
            continue;
        }

        // Get room id before .room extension
        let room_id = path
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .split("_Room_")
            .last()
            .unwrap();
        println!("Room ID: {}", room_id);

        // open file
        let file = std::fs::File::open(path.clone());
        if file.is_err() {
            error!("Error opening file: {:?}", file.err());
            continue;
        }

        let mut file = file.unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();

        let mut room = Room::from_bytes(&bytes);
        room.room_id = room_id.to_string();
        room.save_image();
    }
}
