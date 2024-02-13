mod music_play; // Note: Use an underscore in the module nam
use music_play::play_random_song; // Use functions from the music-play module


use std::{
    fs,
    io::{Write, Result, Read},
    env,
    error::Error
};
use std::fs::File;
use walkdir::WalkDir;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize)]
struct MusicConfig {
    music_directory: String,
    music_list: Vec<String>,
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect(); // Skip the program name
    let debug_mode = args.iter().any(|arg| arg == "-d" || arg == "--debug");

    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print_help();
        return Ok(());
    }

    if debug_mode {
        println!("Debug mode: Enabled");
    }

    // Use the filtered args to find the music directory, excluding flags
    let music_directory_arg = args.iter()
        .find(|arg| !arg.starts_with('-'))
        .map(|s| s.as_str());

    let mut music_config = if let Some(music_directory) = music_directory_arg {
        if debug_mode {
            println!("Config update: Using directory '{}'", music_directory);
        }
        update_config(music_directory)? // update_config no longer takes debug_mode
    } else {
        if debug_mode {
            println!("Config read: Reading existing configuration");
        }
        read_music_config()? // read_music_config no longer takes debug_mode
    };

    if music_config.music_list.is_empty() {
        if debug_mode {
            println!("Updating music list from directory '{}'", music_config.music_directory);
        }
        music_config.music_list = music_array(&music_config.music_directory)?; // music_array no longer takes debug_mode
        save_music_config(&music_config)?; // save_music_config no longer takes debug_mode
    } else if debug_mode {
        println!("Music list loaded with {} songs", music_config.music_list.len());
    }

    if debug_mode {
        println!("Playing random song");
    }
    play_random_song(&music_config.music_list,debug_mode)?; // play_random_song no longer takes debug_mode

    if debug_mode {
        println!("Main function completed");
    }

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

fn print_help() {
    println!("Rustyplayer Help Menu");
    println!("Usage: rustyplayer [OPTIONS] [MUSIC_DIRECTORY]");
    println!("");
    println!("Options:");
    println!("  -h, --help       Display this help menu and exit");
    println!("  -d, --debug      Run the program in debug mode to display additional information and prevents the terminal screen from clearing");
    println!("");
    println!("MUSIC_DIRECTORY is an optional argument. If provided, Rustyplayer will use this directory to update the music library.");
}