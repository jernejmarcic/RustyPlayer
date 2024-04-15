mod music_play;
use music_play::play_random_song;
use std::{
    fs::{self, OpenOptions},
    io::{Result, Write},
    env,
    path::PathBuf,
    collections::HashSet
};
use walkdir::WalkDir;
use serde::{Deserialize, Serialize};
use serde_json;


#[derive(Serialize, Deserialize)]
struct MusicConfig {
   // extension_check: Vec<String>, // Stores user-specified music extensions, all music extensions are enabled by deafult
    music_directories: Vec<String>, // Stores user-specified music directories, empty by default
    music_list: Vec<String>, // Walkdir will put in the absolute paths to all the music files.
}

const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");
const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<()> {
    // Collect command line arguments, skipping the first one (the program name).
    let args: Vec<String> = env::args().skip(1).collect();

    // Check if the debug mode flag ("-d" or "--debug") is present among the arguments.
    let debug_mode = args.iter().any(|arg| arg == "-d" || arg == "--debug");
    if debug_mode { println!("Debug mode: Enabled");}

    // Check for the help flag ("-h" or "--help") and print help information if present.
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print_help(); // Function to display help information.
        return Ok(()); // Exit the program after displaying help.
    }

    // Filter out any flags (arguments starting with '-') and collect the remaining ones as music directories.
    let music_directories: Vec<String> = args.iter()
        .filter(|arg| !arg.starts_with('-'))
        .cloned()
        .collect::<HashSet<_>>() // Use a HashSet to remove duplicate entries.
        .into_iter()
        .collect(); // Convert the HashSet back into a Vec.
    if debug_mode { println!("Using directories: {:?}", music_directories); }

    // If music directories are provided, update the configuration. Otherwise, read the existing configuration.
    let mut music_config = if !music_directories.is_empty() {
        if debug_mode { println!("Config update: Using directories {:?}", music_directories); }
        update_config(&music_directories, debug_mode)? // Update the configuration based on the provided directories.
    } else {
        if debug_mode { println!("Config read: Reading existing configuration"); }
        read_music_config()? // Read the existing configuration.
    };

    // If the music list is empty but directories are specified, update the music list from those directories.
    if music_config.music_list.is_empty() && !music_config.music_directories.is_empty() {
        if debug_mode { println!("Updating music list from directories {:?}", music_config.music_directories); }
        for directory in &music_config.music_directories {
            let music_list = music_array(directory, debug_mode)?; // Collect music files from the directory.
            music_config.music_list.extend(music_list); // Add the collected music files to the music list.
        }
        save_music_config(&music_config)?; // Save the updated configuration.
    } else if debug_mode {
        println!("Music list loaded with {} songs", music_config.music_list.len());
    }

    if debug_mode { println!("Playing random song"); }
    // Play a random song from the music list.
    play_random_song(&music_config.music_list, debug_mode)?;

    if debug_mode {
        println!("Main function completed");
    }

    Ok(())
}


// Function to update the music configuration based on the provided directories.
// It takes a slice of directory paths and a boolean indicating whether debug mode is active.
fn update_config(directories: &[String], debug_mode: bool) -> Result<MusicConfig> {
    // Initialize an empty vector to aggregate music file paths from all directories.
    let mut aggregated_music_list = Vec::new();

    // Iterate over each directory provided in the input.
    for directory in directories {
        // Retrieve the list of music file paths from the current directory.
        // `music_array` is assumed to be a function that reads the directory and returns a Vec<String> of music file paths.
        let music_list = music_array(directory, debug_mode)?;
        // Extend the aggregated music list with the music files from the current directory.
        aggregated_music_list.extend(music_list);
    }

    // Create a new instance of MusicConfig with the provided directories and the aggregated music list.
    let music_config = MusicConfig {
        music_directories: directories.to_vec(), // Convert the slice of directories to a Vec.
        music_list: aggregated_music_list, // The aggregated list of music file paths.
    };

    // Save the updated music configuration to a file or database.
    save_music_config(&music_config)?;

    // Return the updated music configuration.
    Ok(music_config)
}


// Function to save the music configuration to a JSON file.
fn save_music_config(music_config: &MusicConfig) -> Result<()> {
    // Determine the path for the configuration directory.
    let config_dir = config_path();
    // If the configuration directory does not exist, create it along with any necessary parent directories.
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }
    // Construct the path for the configuration file within the configuration directory.
    let config_file = config_dir.join("playlist_config.json");

    // Open the configuration file for writing. If the file does not exist, it will be created.
    // If it does exist, its contents will be truncated before writing.
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(config_file)?;

    // Serialize the music configuration struct to a JSON string.
    let config_string = serde_json::to_string(music_config)?;
    // Write the serialized JSON string to the configuration file.
    file.write_all(config_string.as_bytes())?;
    // If all operations succeed, return an Ok result.
    Ok(())
}


// Function to determine the configuration file path.
fn config_path() -> PathBuf {
    // Retrieve the current user's name from the environment variables.
    let user = env::var("USER").expect("USER environment variable not set");
    // Construct the path to the configuration file based on the user's home directory.
    PathBuf::from(format!("/home/{}/.config/rustyplayer", user))
}

// Function to read the music configuration from a JSON file.
fn read_music_config() -> Result<MusicConfig> {
    // Use the `config_path` function to determine the full path to the configuration file.
    let config_file = config_path().join("playlist_config.json");
    // Attempt to read the configuration file as a string.
    match fs::read_to_string(config_file) {
        // If successful, deserialize the JSON string into a `MusicConfig` struct.
        Ok(config_string) => serde_json::from_str(&config_string).map_err(From::from),
        // If the file cannot be read, return a default `MusicConfig` struct.
        Err(_) => Ok(MusicConfig {
            music_directories: Vec::new(), // Start with an empty list of music directories.
            music_list: Vec::new(), // Start with an empty list of music files.
        }),
    }
}


// Function to collect all music files from a given directory path.
fn music_array(music_path: &str, debug_mode: bool) -> Result<Vec<String>> {
    let mut music_list = Vec::new(); // Vector to store paths to all valid music files.
    let mut non_music_files_count = 0; // Counter for files skipped because they're not recognized as music files.
    // List of file extensions considered to be music files.
    let music_extensions = ["mp3", "flac", "wav", "opus"];

    // Walk through the directory at `music_path`, including all subdirectories.
    for entry in WalkDir::new(music_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() { // Ensure we're dealing with a file.
            match path.extension().and_then(|ext| ext.to_str()) {
                // If the file extension is in our list of music file extensions, add it to `music_list`.
                Some(ext) if music_extensions.contains(&ext) => {
                    music_list.push(path.to_string_lossy().into_owned());
                },
                // If the file extension is not recognized as a music file, increment `non_music_files_count`.
                _ => non_music_files_count += 1,
            }
        }
    }

    // If debug mode is enabled, print the number of non-music files that were skipped.
    if debug_mode { println!("Skipped {} non-music files", non_music_files_count); }
    Ok(music_list) // Return the list of music file paths.
}

// Well it's a help message I don't think an explanation is needed
fn print_help() {
    println!("{} Help Menu",PACKAGE_NAME);
    println!("Usage: {} [OPTIONS] [MUSIC_DIRECTORY]", PACKAGE_NAME);
    println!("Usage extended: {} [OPTIONS] [MUSIC_DIRECTORY 1] [MUSIC_DIRECTORY 2]", PACKAGE_NAME);
    println!("");
    println!("Options:");
    println!("  -h, --help       Display this help menu and exit");
    println!("  -d, --debug      Run the program in debug mode to display additional information and prevents the terminal screen from clearing");
    println!("");
    println!("MUSIC_DIRECTORY is an optional argument, there can be multiple. If provided, {} will use this directory to update the music library.", PACKAGE_NAME);
    // println!("Your current music path is set to: {}",music_directory);
    println!("Configuration is located at: {}", config_path().display());
    println!("_________________________________________________");
    println!("Version: {}",PACKAGE_VERSION)
}