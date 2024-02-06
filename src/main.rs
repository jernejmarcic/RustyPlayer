use clap::Command; // Updated from `clap::App` to `clap::Command`
use clap::Arg;
use rand::seq::SliceRandom; // Ensure `rand` is added to your `Cargo.toml`
use serde::{Serialize, Deserialize};
use serde_json::Result as JsonResult;
use std::fs;
use std::io::BufReader;
use std::path::PathBuf;
use std::fs::File;
use walkdir::WalkDir;
use rodio::{Decoder, OutputStream, Sink};

const CONFIG_FILE_PATH: &str = "/path/to/your/music_player.conf";

#[derive(Serialize, Deserialize)]
struct Config {
    music_directory: String,
}

fn main() -> JsonResult<()> {
    let matches = Command::new("Music Player") // Updated to `Command::new`
        .arg(Arg::new("directory")
            .help("Sets the music directory")
            .required(true)
            .index(1)) // Positional argument remains the same
        .get_matches();

    let music_dir = matches.get_one::<String>("directory").expect("required argument"); // Updated to `get_one`

    let config_path = PathBuf::from(CONFIG_FILE_PATH);
    save_path_to_config(&config_path, music_dir)?;

    let music_files = find_music_files(PathBuf::from(music_dir))?;
    play_music_randomly(music_files)?;

    Ok(())
}

fn save_path_to_config(config_path: &PathBuf, path: &str) -> JsonResult<()> {
    let config = Config {
        music_directory: path.to_string(),
    };
    fs::write(config_path, serde_json::to_string(&config)?)?;
    Ok(())
}

fn find_music_files(path: PathBuf) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut music_files = Vec::new();
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            music_files.push(entry.into_path());
        }
    }
    Ok(music_files)
}

fn play_music_randomly(files: Vec<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = rand::thread_rng();
    let mut shuffled_files = files.clone();
    shuffled_files.shuffle(&mut rng);

    for file_path in shuffled_files {
        play_audio(file_path)?;
    }

    Ok(())
}

fn play_audio(audio_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let file = BufReader::new(File::open(audio_path)?);
    let source = Decoder::new(file)?;
    let sink = Sink::try_new(&stream_handle)?;
    sink.append(source);
    sink.sleep_until_end();
    Ok(())
}
