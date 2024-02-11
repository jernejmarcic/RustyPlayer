use std::{fs::File,
          io::BufReader,
          path::PathBuf,
          time::Duration,
          sync::mpsc::Sender,
          path::Path,
          process::Command,
          sync::{Arc, Mutex,
                 atomic::{AtomicBool, Ordering}},
          thread
};
use std::thread::sleep;
use rand::Rng;
use rodio::{Decoder, OutputStream, Sink};
use audiotags::Tag;
// use termion::event::{Key, parse_event};
// use dbus::{blocking::Connection, channel::MatchingReceiver, message::MatchRule};
// use dbus::blocking::stdintf::org_freedesktop_dbus::EmitsChangedSignal::False;
use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, PlatformConfig};

static IS_PAUSED: AtomicBool = AtomicBool::new(false); // Start in a play state
static SHOULD_SKIP: AtomicBool = AtomicBool::new(false);
static SHOULD_PLAY_PREVIOUS: AtomicBool = AtomicBool::new(false);

pub(crate) fn play_random_song(music_list: &[String]) -> std::io::Result<()> {
    if music_list.is_empty() {
        println!("No songs found in the specified directory.");
        return Ok(());
    }

    let mut rng = rand::thread_rng();
    let mut played_songs = Vec::new();

    let mut last_paused_state = IS_PAUSED.load(Ordering::SeqCst);

    loop {

        let current_paused_state = IS_PAUSED.load(Ordering::SeqCst);
        let randint = rng.gen_range(0..music_list.len());

        // Your actual logic goes here.
        played_songs.push(randint);  // Track played songs
        println!("Playing song: {}", music_list[randint]);
     //   println!("Played songs index: {:?}", played_songs);
        // Get a output stream handle to the default physical sound device
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let file = BufReader::new(File::open(&music_list[randint]).unwrap());
        let sink = Sink::try_new(&stream_handle).unwrap();
        let tag = Tag::new().read_from_path(&music_list[randint]).unwrap();
        let title = tag.title().unwrap_or_else(|| "Unknown".into());
        let album_cover = tag.album_cover();
        let artists = tag
            .artists()
            .map(|a| a.join(", "))
            .unwrap_or_else(|| "Unknown".into());
        let album = tag.album_title().unwrap_or_else(|| "Unknown".into());

// Add a dummy source of the sake of the example.
        let source = Decoder::new(file).unwrap();
        sink.append(source);

        terminal_ui(&music_list, randint, title, album, artists.clone());

        let played_songs_clone = played_songs.clone();  // Clone played_songs for the closure
        let music_list_clone = music_list;

        #[cfg(not(target_os = "windows"))]
            let hwnd = None;

        #[cfg(target_os = "windows")]
            let hwnd = {
            use raw_window_handle::windows::WindowsHandle;

            let handle: WindowsHandle = unimplemented!();
            Some(handle.hwnd)
        };

        let config = PlatformConfig {
            dbus_name: "rustyplayer",
            display_name: "Rusty Player",
            hwnd,
        };

        let mut controls = MediaControls::new(config).unwrap();

        // The closure must be Send and have a static lifetime.
        controls
            .attach(
                move |event: MediaControlEvent| match event {
                MediaControlEvent::Play => {
                    println!("Play event received");
                    let current_state = IS_PAUSED.load(Ordering::SeqCst);
                    IS_PAUSED.store(!current_state, Ordering::SeqCst);  // Toggle the state
                },

                MediaControlEvent::Pause => {
                    println!("Pause event received");
                    // Logic to pause the music
                    let current_state = IS_PAUSED.load(Ordering::SeqCst);
                    IS_PAUSED.store(!current_state, Ordering::SeqCst);  // Toggle the state

                },
                MediaControlEvent::Toggle => {
                    println!("Toggle event recieved");
                    // Toggle logic here
                    let current_state = IS_PAUSED.load(Ordering::SeqCst);
                    IS_PAUSED.store(!current_state, Ordering::SeqCst);  // Toggle the state
                },

                MediaControlEvent::Next => {
                    println!("Next event received");
                    // Logic to skip to the next track
                    // You might need to signal your playback loop to move to the next song
                    SHOULD_SKIP.store(true, Ordering::SeqCst);

                },
                    MediaControlEvent::Previous => {
                        println!("Previous button clicked");
                        if played_songs_clone.len() >= 2 {
                            let previous_index = played_songs_clone[played_songs_clone.len() - 2];

                            // Call play_previous_track with the index of the previous song
                            play_previous_track(previous_index, &music_list_clone, &sink);
                        } else {
                            println!("No previous song to play or it's the first song.");
                        }
                    },
                // Add more event handlers as needed
                _ => println!("Event received: {:?}", event),
            })
            .unwrap();



        // Update the media metadata.
        controls
            .set_metadata(MediaMetadata {
                title: Some(title),
                artist: Some(&*artists),
                album: Some(album),



                ..Default::default()
            })
            .unwrap();


        while !sink.empty() && !SHOULD_SKIP.load(Ordering::SeqCst) {
            // Check and handle play/pause state...
            let current_paused_state = IS_PAUSED.load(Ordering::SeqCst);
            if current_paused_state {
                sink.pause();
            } else {
                sink.play();
            }

            thread::sleep(Duration::from_millis(100));
        }

        // Check again for skip in case it was set during playback
        if SHOULD_SKIP.load(Ordering::SeqCst) {
            SHOULD_SKIP.store(false, Ordering::SeqCst);  // Reset the flag
            println!("Skipping to the next track...");
            // No need for 'continue;' here as it's the end of the loop
            continue;
        }

        sink.sleep_until_end();


// The sound plays in a separate thread. This call will block the current thread until the sink
// has finished playing all its queued sounds.

    }






    Ok(())
}


fn extract_cover_from_flac(flac_path: &String) -> Result<(), std::io::Error> {
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
fn display_full_image_with_chafa(image_path: &str) -> Result<(), std::io::Error> {
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

fn terminal_ui(music_list: &[String], randint: usize, title: &str, artists: &str, album: String) {
    let flac_checker = ".flac";


    if music_list[randint].contains(flac_checker) {
        extract_cover_from_flac(&music_list[randint]).unwrap();
        display_full_image_with_chafa("cover.jpg").unwrap();
    } else {

    }
    println!("Title:     {} \r", title);
    println!("Artists:   {} \r", artists);
    println!("Album:     {} \r", album);
    //println!("Duration: {:?} \r", duration);

}
/*
fn play_previous_track(played_songs_clone: Vec<usize>, music_list: &[String], sink: &Sink) {
    if played_songs_clone.len() >= 2 {
        // Use the second-last element for the previous song
        let previous_index = played_songs_clone[played_songs_clone.len() - 2];

        // Logic to play the previous song
        println!("Playing previous song: {}", music_list[previous_index]);
        let file = BufReader::new(File::open(&music_list[previous_index]).unwrap());
        let source = Decoder::new(file).unwrap();

        // Stop the current playing song and clear the Sink
        sink.stop();
        // It's not necessary to recreate the Sink, just append the new source
        sink.append(source);
        // Play the song
        sink.play();
    } else {
        println!("No previous song to play or it's the first song.");
    }
}
*/
fn play_previous_track(previous_index: usize, music_list: &[String], sink: &Sink) {
    println!("Playing previous song: {}", music_list[previous_index]);
    let file = BufReader::new(File::open(&music_list[previous_index]).unwrap());
    let source = Decoder::new(file).unwrap();

    sink.stop();
    sink.append(source);
    sink.play();
}
