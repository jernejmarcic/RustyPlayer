mod music_play;
use music_play::play_random_song;

use std::{
    fs::{self, OpenOptions},
    io::{Result, Write},
    env,
    path::PathBuf,
};
use walkdir::WalkDir;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize)]
struct MusicConfig {
    music_directory: String,
    music_list: Vec<String>,
}

const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");
const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let debug_mode = args.iter().any(|arg| arg == "-d" || arg == "--debug");

    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print_help();
        return Ok(());
    }

    let music_directory_arg = args.iter().find(|arg| !arg.starts_with('-')).map(|s| s.as_str());

    if debug_mode { println!("Debug mode: Enabled"); }

    let mut music_config = if let Some(music_directory) = music_directory_arg {
        if debug_mode { println!("Config update: Using directory '{}'", music_directory); }
        update_config(music_directory)?
    } else {
        if debug_mode { println!("Config read: Reading existing configuration"); }
        read_music_config()?
    };

    if music_config.music_list.is_empty() {
        if debug_mode { println!("Updating music list from directory '{}'", music_config.music_directory); }
        music_config.music_list = music_array(&music_config.music_directory)?;
        save_music_config(&music_config)?;
    } else if debug_mode { println!("Music list loaded with {} songs", music_config.music_list.len()); }

    if debug_mode { println!("Playing random song"); }

    play_random_song(&music_config.music_list, debug_mode, /*&config_path()*/)?;

    if debug_mode { println!("Main function completed"); }

    Ok(())
}

fn update_config(music_path: &str) -> Result<MusicConfig> {
    let music_list = music_array(music_path)?;
    let music_config = MusicConfig { music_directory: music_path.to_string(), music_list };
    save_music_config(&music_config)?;
    Ok(music_config)
}

fn save_music_config(music_config: &MusicConfig) -> Result<()> {
    let config_dir = config_path();
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }
    let config_file = config_dir.join("playlist_config.json");

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(config_file)?;

    let config_string = serde_json::to_string(music_config)?;
    file.write_all(config_string.as_bytes())?;
    Ok(())
}

fn config_path() -> PathBuf {
    let user = env::var("USER").expect("USER environment variable not set");
    PathBuf::from(format!("/home/{}/.config/rustyplayer", user))
}

fn read_music_config() -> Result<MusicConfig> {
    let config_file = config_path().join("playlist_config.json");
    match fs::read_to_string(config_file) {
        Ok(config_string) => serde_json::from_str(&config_string).map_err(From::from),
        Err(_) => Ok(MusicConfig { music_directory: String::new(), music_list: Vec::new() }),
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
    Ok(music_list) // Directly return an Ok variant of Result with music_list
}

fn print_help(): {
    println!("{} Help Menu",PACKAGE_NAME);
    println!("Usage: {} [OPTIONS] [MUSIC_DIRECTORY]", PACKAGE_NAME);
    println!("");
    println!("Options:");
    println!("  -h, --help       Display this help menu and exit");
    println!("  -d, --debug      Run the program in debug mode to display additional information and prevents the terminal screen from clearing");
    println!("");
    println!("MUSIC_DIRECTORY is an optional argument. If provided, {} will use this directory to update the music library.", PACKAGE_NAME);
    println!("Your current music path is set to: {}",music_directory);
    println!("Configuration is located at: {}", config_path().display());
    println!("_________________________________________________");
    println!("Version: {}",PACKAGE_VERSION)
}