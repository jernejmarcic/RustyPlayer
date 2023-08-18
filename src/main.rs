use audiotags::Tag;
use dbus::blocking::Connection;
use dbus::message::MatchRule;
use device_query::{DeviceQuery, DeviceState, Keycode};
use indicatif::ProgressBar;
use rand::rngs::ThreadRng;
use rand::Rng;
use rodio::source::{SineWave, Source};
use rodio::{Decoder, OutputStream, Sink};
use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, PlatformConfig};
use std::fs::read;
use std::fs::File;
use std::io::BufReader;
use std::io::{self, empty};
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use walkdir::WalkDir;
use walkdir::*;

//struct PlayerControl {
//
//    rx: Receiver<MediaControlEvent>,
//   sink: Arc<Mutex<Sink>>,  // Use an Arc<Mutex<Sink>> to share the Sink safely across threads
//}

//impl PlayerControl {
//    pub fn new(sink: Sink) -> Self {
//        let config = PlatformConfig {
//            dbus_name: "oxiplayer",
//            display_name: "Oxiplayer",
//            hwnd: None,
//       };
//
//        let mut controls = MediaControls::new(config).unwrap();
//       let (tx, rx) = channel();
//
//        controls
//            .attach(move |event| tx.send(event).unwrap())
//            .unwrap();
//
//        Self {
//           controls,
//            rx,
//            sink: Arc::new(Mutex::new(sink)),
//        }
//    }
//
//    pub fn listen(&self) {
//        while let Ok(event) = self.rx.recv() {
//            match event {
//                MediaControlEvent::Toggle => {
//                    let mut sink = self.sink.lock().unwrap();
//                    sink.pause()
//                }
//                MediaControlEvent::Next => {
//                //;
//                }
//                MediaControlEvent::Previous => {
//
//                }
//                // ... Handle other events as required
//                _ => {}
//            }
//        }
//    }
//}

fn main() {
    let paths: Vec<_> = WalkDir::new("/home/jernej/Music/Playlist")
        .into_iter()
        .filter_map(Result::ok)
        .map(|e| e.path().to_path_buf())
        .collect();

    let mut played_songs: Vec<i32> = Vec::new();

    let mut rng = rand::thread_rng();

    // Create the output stream and sink here
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    // Pass the sink to the PlayerControl::new method
    //    let player_control = PlayerControl::new(sink);
    //    start_mpris_listener(player_control);

    looper(&paths, &mut played_songs, &mut rng);
}

fn looper(paths: &Vec<PathBuf>, played_songs: &mut Vec<i32>, rng: &mut ThreadRng) {
    loop {
        let randint = rng.gen_range(0..paths.len());
        played_songs.push(randint as i32);
        println!("Song numbers played: {:?}", played_songs);
        play_random_song(paths, randint, rng);
    }
}

fn play_random_song(paths: &Vec<PathBuf>, randint: usize, rng: &mut ThreadRng) {
    clearscreen::clear().expect("failed to clear screen");
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    let tag = Tag::new().read_from_path(&paths[randint]).unwrap();
    let title = tag.title().unwrap_or_else(|| "Unknown".into());
    let artists = tag
        .artists()
        .map(|a| a.join(", "))
        .unwrap_or_else(|| "Unknown".into());
    let album = tag.album_title().unwrap_or_else(|| "Unknown".into());

    println!("Title:     {}", title);
    println!("Artists:   {}", artists);
    println!("Album:     {}", album);

    let file = BufReader::new(File::open(&paths[randint]).unwrap());
    let source = Decoder::new(file).unwrap();
    sink.append(source);

    let mut stdout = io::stdout().into_raw_mode().unwrap();
    let stdin = io::stdin();
    let mut keys = stdin.keys();
    loop {
        if sink.empty() {
            break;
        }

        let key = match keys.next() {
            Some(result) => match result {
                Ok(k) => k,
                Err(_) => continue, // handle the error as you see fit
            },
            None => continue,
        };

        match key {
            Key::Char('p') => {
                // Play/Pause toggle
                if sink.is_paused() {
                    sink.play();
                } else {
                    sink.pause();
                }
            }
            Key::Char('n') => {
                return;
            }
            Key::Char('b') => {
                // Your logic for the 'b' key
                break;
            }
            _ => continue,
        }
    }

    // The sound plays in a separate thread. This call will block the current thread until the sink
    // has finished playing all its queued sounds.
    sink.sleep_until_end();
}

//fn start_mpris_listener(player_control: PlayerControl) {
//    thread::spawn(move || {
//        player_control.listen();
//   });
//}
