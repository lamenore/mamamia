use std::io::Read;

use log::error;
pub mod constants;
pub mod shapes;
mod types;
use anyhow::anyhow;
use types::room::Room;

fn main() -> Result<(), anyhow::Error> {
    // Parse command-line arguments
    let args: Vec<String> = std::env::args().collect();

    let should_save_images = args.contains(&"-i".to_string());
    let should_save_slopes = args.contains(&"-s".to_string());
    let should_save_breakables = args.contains(&"-b".to_string());

    let directory_path = std::path::Path::new("./bins");
    if !directory_path.exists() {
        println!(
            "Directory not found: {}",
            std::path::absolute(directory_path).unwrap().display()
        );
        return Err(anyhow!("Directory not found"));
    }

    // Retrieve all files in the directory
    let directory_entries = std::fs::read_dir(directory_path)?;
    for entry in directory_entries {
        let entry = entry?;
        let file_path = entry.path();

        // Check if the file is a .room file
        if !file_path.is_file() || file_path.extension().and_then(|ext| ext.to_str()) != Some("room") {
            continue;
        }

        // Extract room ID from file name
        let room_id = file_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .and_then(|name| name.split("_Room_").last())
            .unwrap_or_default();

        // Open the file
        let mut file = std::fs::File::open(&file_path)?;
        let mut file_content = Vec::new();
        if let Err(err) = file.read_to_end(&mut file_content) {
            error!("Failed to read file: {}", err);
            continue;
        }

        let mut room = Room::from_bytes(&file_content);
        room.room_id = room_id.to_string();

        if should_save_images {
            //println!("Saving image for: {}", room_id);
            room.save_image()?;
        }
        if should_save_slopes {
            //println!("Saving slopes for: {}", room_id);
            room.save_slopes();
        }
        if should_save_breakables {
            //println!("Saving breakables for: {}", room_id);
            room.save_breakables();
        }
    }
    Ok(())
}
