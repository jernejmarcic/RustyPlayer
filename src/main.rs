use audiotags::Tag;
use rand::rngs::ThreadRng;
use rand::Rng;
use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use termion::event::Key;
use termion::input::TermRead;
use walkdir::WalkDir;
use std::sync::mpsc::{self, TryRecvError};
use std::{thread, time::Duration};
use std::io;
use termion::raw::IntoRawMode;
use std::process::Command;
use std::path::Path;


fn main() {
    // Enable raw mode
    let _stdout = io::stdout().into_raw_mode().unwrap();

    let paths: Vec<_> = WalkDir::new("/home/jernej/Music/Playlist")
        .into_iter()
        .filter_map(Result::ok)
        .map(|e| e.path().to_path_buf())
        .collect();

    let mut played_songs: Vec<i32> = Vec::new();
    let mut rng = rand::thread_rng();

    let key_channel = spawn_stdin_channel();
    looper(&paths, &mut played_songs, &mut rng, &key_channel);
}


fn looper(
    paths: &Vec<PathBuf>,
    played_songs: &mut Vec<i32>,
    rng: &mut ThreadRng,
    key_channel: &mpsc::Receiver<Key>
) {
    loop {
        let randint = rng.gen_range(0..paths.len());
        played_songs.push(randint as i32);
        // println!("Song numbers played: {:?} \r", played_songs);
        play_random_song(paths, randint, rng, key_channel);
    }
}

fn play_random_song(
    paths: &Vec<PathBuf>,
    randint: usize,
    rng: &mut ThreadRng,
    key_channel: &mpsc::Receiver<Key>
) {
    clearscreen::clear().expect("failed to clear screen");
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    let tag = Tag::new().read_from_path(&paths[randint]).unwrap();

    // Extract and display the album cover
    extract_cover_from_flac(&paths[randint]).unwrap();
    display_image_as_ascii("cover.jpg").unwrap();
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    let tag = Tag::new().read_from_path(&paths[randint]).unwrap();
    let title = tag.title().unwrap_or_else(|| "Unknown".into());
    let artists = tag
        .artists()
        .map(|a| a.join(", "))
        .unwrap_or_else(|| "Unknown".into());
    let album = tag.album_title().unwrap_or_else(|| "Unknown".into());

    println!("Title:     {} \r", title);
    println!("Artists:   {} \r", artists);
    println!("Album:     {} \r", album);

    let file = BufReader::new(File::open(&paths[randint]).unwrap());
    let source = Decoder::new(file).unwrap();
    sink.append(source);

    while !sink.empty() == true {
        match key_channel.try_recv() {
            Ok(key) => handle_key_input(key, &sink),
            Err(TryRecvError::Empty) => {},
            Err(TryRecvError::Disconnected) => {
                eprintln!("Key input channel disconnected");
            }

        }
        thread::sleep(Duration::from_millis(100));
    }
}


fn handle_key_input(key: Key, sink: &Sink) {
    let _stdout = io::stdout().into_raw_mode().unwrap();
    match key {
        Key::Char('p') => {
            if sink.is_paused() {
                sink.play();
            } else {
                sink.pause();
            }
        },
        Key::Char('n') => {
            // skip to next song logic
            sink.stop();
        },
        Key::Char('b') => {
            // go back to previous song logic
            // depending on your implementation
        },
        _ => {}
    }
}

fn spawn_stdin_channel() -> mpsc::Receiver<Key> {
    let (tx, rx) = mpsc::channel::<Key>();
    thread::spawn(move || {
        let mut stdin = termion::async_stdin().keys();
        loop {
            if let Some(Ok(key)) = stdin.next() {
                tx.send(key).unwrap();
            } else {
                // You could add a small sleep here if you wish to prevent busy-waiting
                thread::sleep(Duration::from_millis(10));
            }
        }
    });
    rx
}
fn display_image_as_ascii(image_path: &str) -> Result<(), std::io::Error> {
    let output = Command::new("chafa")
        .arg("-s")
        .arg("80x25") // Adjust the size as necessary
        .arg(image_path)
        .output()?;

    if output.status.success() {
        print!("{} \r", String::from_utf8_lossy(&output.stdout));
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to display image",
        ))
    }
}

fn extract_cover_from_flac(flac_path: &Path) -> Result<(), std::io::Error> {
    let output_path = "cover.jpg"; // You can customize this
    let output = Command::new("metaflac")
        .arg("--export-picture-to")
        .arg(output_path)
        .arg(flac_path)
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to extract album cover",
        ))
    }
}

