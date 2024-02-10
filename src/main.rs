

mod music_play; // Note: Use an underscore in the module name

use music_play::play_random_song; // Use functions from the music-play module


use std::{
    fs,
    io::{BufReader, Write, Result, Read},
    path::PathBuf,
    env,
    sync::mpsc,
    thread,
    time::Duration
};
use std::fs::File;
use walkdir::WalkDir;
use rand::Rng;
use rodio::{Decoder, OutputStream, Sink};
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize)]
struct MusicConfig {
    music_directory: String,
    music_list: Vec<String>,
}

fn main() -> Result<()> {
    println!("Program started");

    let args: Vec<String> = env::args().collect();
    let mut music_config = if args.len() > 1 {
        let music_directory = &args[1];
        update_config(music_directory)?
    } else {
        read_music_config()?
    };

    if music_config.music_list.is_empty() {
        println!("Updating music list from directory");
        music_config.music_list = music_array(&music_config.music_directory)?;
        save_music_config(&music_config)?;
    }

    println!("Music directory set to: {}", music_config.music_directory);
    // println!("Music list contains: {:?}", music_config.music_list);

    play_random_song(&music_config.music_list)?;

    println!("Main function completed");
    Ok(())
}

fn update_config(music_path: &str) -> Result<MusicConfig> {
    let music_list = music_array(music_path)?;
    let music_config = MusicConfig {
        music_directory: music_path.to_string(),
        music_list,
    };
    save_music_config(&music_config)?;
    Ok(music_config)
}

fn save_music_config(music_config: &MusicConfig) -> Result<()> {
    let config_path = "music_array.conf";
    let config_string = serde_json::to_string(music_config)?;
    fs::write(config_path, config_string)?;
    Ok(())
}

fn read_music_config() -> Result<MusicConfig> {
    let config_path = "music_array.conf";
    match fs::read_to_string(config_path) {
        Ok(config_string) => {
            let music_config: MusicConfig = serde_json::from_str(&config_string)?;
            Ok(music_config)
        },
        Err(_) => Ok(MusicConfig {
            music_directory: String::new(),
            music_list: Vec::new(),
        }),
    }
}

fn music_array(music_path: &str) -> Result<Vec<String>> {
    let mut music_list = Vec::new();
    for entry in WalkDir::new(music_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            music_list.push(path.to_string_lossy().into_owned());
        }
    }
    Ok(music_list)
}
