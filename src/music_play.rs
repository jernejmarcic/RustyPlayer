use std::{fs::File,
          io::BufReader,
          //path::PathBuf,
          time::Duration,
          //sync::mpsc::Sender,
          //path::Path,
          process::Command,
          sync::{/*Arc, Mutex,*/
                 atomic::{AtomicBool, Ordering}},
          thread,
};
// use std::path::{PathBuf};
//use std::io::Write;
//use std::thread::sleep;
use rand::Rng;
use rodio::{Decoder, OutputStream, Sink};
use audiotags::{Tag};
// use termion::event::{Key, parse_event};
// use dbus::{blocking::Connection, channel::MatchingReceiver, message::MatchRule};
// use dbus::blocking::stdintf::org_freedesktop_dbus::EmitsChangedSignal::False;
use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, PlatformConfig};


// This just basically changes the duration from Option<Duration> to Option<f64> which is
fn convert_to_duration(option_seconds: Option<f64>) -> Option<Duration> {
    option_seconds.map(|secs| Duration::from_secs_f64(secs))
}


static IS_PAUSED: AtomicBool = AtomicBool::new(false);
// Start in a play state
static LAST_PAUSED_STATE: AtomicBool = AtomicBool::new(false);
static SHOULD_SKIP: AtomicBool = AtomicBool::new(false);
//static SHOULD_PLAY_PREVIOUS: AtomicBool = AtomicBool::new(false);


pub(crate) fn play_random_song(music_list: &[String], debug_mode: bool /*, config_path: &PathBuf*/) -> std::io::Result<()> {
    if music_list.is_empty() {
        println!("No songs found in the specified directory.");
        return Ok(());
    }

    let mut played_songs: Vec<usize> = Vec::new();
    music_player(music_list, debug_mode, &mut played_songs, /*&mut rng*/);
    Ok(())
}

fn music_player(music_list: &[String], debug_mode: bool, played_songs: &mut Vec<usize>, /* rng: &mut rand::ThreadRng */) {
    let mut rng = rand::thread_rng();
//    let mut last_paused_state = IS_PAUSED.load(Ordering::SeqCst);
    //       let current_paused_state = IS_PAUSED.load(Ordering::SeqCst);
    let randint = rng.gen_range(0..music_list.len());
    // Your actual logic goes here.
    played_songs.push(randint);  // Track played songs
    if debug_mode { println!("Playing song number: {}", randint) }
    if debug_mode { println!("Song numbers played: {:?}", played_songs) }
    if debug_mode { println!("Playing song file: {}", music_list[randint]); }
    // println!("Played songs index: {:?}", played_songs);
    // Get an output stream handle to the default physical sound device
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let file = BufReader::new(File::open(&music_list[randint]).unwrap());
    let sink = Sink::try_new(&stream_handle).unwrap();

    let tag = Tag::new().read_from_path(&music_list[randint]).unwrap();
    let title = tag.title().unwrap_or_else(|| "Unknown".into());
    //       let album_cover = tag.album_cover();
    let duration_seconds: Option<f64> = tag.duration();  // Example duration in miliseconds
    let duration: Option<Duration> = convert_to_duration(duration_seconds);
    let artists = tag
        .artists()
        .map(|a| a.join(", "))
        .unwrap_or_else(|| "Unknown".into());
    let album = tag.album_title().unwrap_or_else(|| "Unknown".into());


// Add a dummy source of the sake of the example.
    let source = Decoder::new(file).unwrap();
    sink.append(source);


    // Construct the path to the directory where the .jpg files are located


    let cover_output_path = format!("/tmp/{}.jpg", album);
    let cover_output_path_clone = cover_output_path.clone();
    if debug_mode { println!("Cover export path is: {}", cover_output_path) }
    terminal_ui(&music_list, randint, title, album, artists.clone(), debug_mode, cover_output_path_clone);

    //      let played_songs_clone = played_songs.clone();  // Clone played_songs for the closure
    //     let music_list_clone = music_list;

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
    // let played_songs_clone = Arc::clone(&played_songs);
    controls
        .attach(
            move |event: MediaControlEvent| match event {
                MediaControlEvent::Play => {
                    if debug_mode { println!("{:?} event received via MPRIS", event) }
                    let current_state = IS_PAUSED.load(Ordering::SeqCst);
                    IS_PAUSED.store(!current_state, Ordering::SeqCst);  // Toggle the state
                }

                MediaControlEvent::Pause => {
                    if debug_mode { println!("{:?} event received via MPRIS", event) }
                    // Logic to pause the music
                    let current_state = IS_PAUSED.load(Ordering::SeqCst);
                    IS_PAUSED.store(!current_state, Ordering::SeqCst);  // Toggle the state
                }
                MediaControlEvent::Toggle => {
                    if debug_mode { println!("{:?} event received via MPRIS", event) }
                    // Toggle logic here
                    let current_state = IS_PAUSED.load(Ordering::SeqCst);
                    IS_PAUSED.store(!current_state, Ordering::SeqCst);  // Toggle the state
                }

                MediaControlEvent::Next => {
                    if debug_mode { println!("{:?} event received via MPRIS", event) }
                    // Logic to skip to the next track
                    // You might need to signal your playback loop to move to the next song
                    SHOULD_SKIP.store(true, Ordering::SeqCst);
                }

                MediaControlEvent::Previous => {
                    if debug_mode { println!("{:?} event received via MPRIS", event) }
                    println!("Well you clicked {:?} but I didn't really code that in yet cuz I can't be bothered to LOL", event);

                    // TODO: Well make it work. LOL
                    // If only it was not so FUCKING HARD.
                }


                // Add more event handlers as needed
                _ => println!("Event received: {:?}. If you see this message contact me I probably just haven't added support yet for it", event),
            })
        .unwrap();


    // Update the media metadata.
    controls
        .set_metadata(MediaMetadata {
            title: Some(title),
            artist: Some(&*artists),
            album: Some(album),
            duration: duration,
            cover_url: Some(&*format!("file://{}", cover_output_path)),
            ..Default::default()
        })
        .unwrap();


    while !sink.empty() && !SHOULD_SKIP.load(Ordering::SeqCst) {
        // Check and handle play/pause state...
        let current_paused_state = IS_PAUSED.load(Ordering::SeqCst);
        if current_paused_state != LAST_PAUSED_STATE.load(Ordering::SeqCst) {
            if current_paused_state {
                if debug_mode { println!("Attempting to pause") }
                sink.pause();
                if debug_mode { println!("Track should be paused") }
                // println!("Paused");
            } else if !current_paused_state {  // Changed from 'else' to 'else if' to explicitly check the condition
                if debug_mode { println!("Attempting to resume/play") }
                sink.play();
                if debug_mode { println!("Track should be resumed") }
                // println!("Play");
            }
            // Update the last paused state to the current state
            LAST_PAUSED_STATE.store(current_paused_state, Ordering::SeqCst);
        }


        thread::sleep(Duration::from_millis(50));
    }

    // Check again for skip in case it was set during playback
    if SHOULD_SKIP.load(Ordering::SeqCst) {
        SHOULD_SKIP.store(false, Ordering::SeqCst);  // Reset the flag
        println!("Attempting to skip to the next track...");
        // Well now the function calls itself so umm even more recursion
        music_player(music_list, debug_mode, played_songs);
    }

    sink.sleep_until_end();
    // This should hopefully make the thing restart when the song is finished
    music_player(music_list, debug_mode, played_songs);

// The sound plays in a separate thread. This call will block the current thread until the sink
// has finished playing all its queued sounds.
}


fn extract_cover_from_flac(flac_path: &str, cover_output_path: String) -> Result<(), std::io::Error> {
    let output = Command::new("metaflac")
        .arg("--export-picture-to")
        .arg(&cover_output_path)
        .arg(flac_path)
        .output();

    match output {
        Ok(output) if output.status.success() => Ok(()),
        Ok(output) => {
            // Print stderr to get more insight into the error
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("metaflac command failed: {}", stderr);
            Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to extract album cover"))
        }
        Err(e) => Err(e),
    }
}

fn display_full_image_with_chafa(image_path: String) -> Result<(), std::io::Error> {
    let output = Command::new("chafa")
        .arg("-s")
        .arg("70x25") // TODO: Make this configurable by the user and save it as a parameter in a config
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

fn terminal_ui(music_list: &[String], randint: usize, title: &str, artists: &str, album: String, debug_mode: bool, cover_output_path: String) {
    if !debug_mode { clearscreen::clear().expect("Failed to clear screen"); }

    let flac_checker = ".flac";

    if music_list[randint].contains(flac_checker) {
        match extract_cover_from_flac(&music_list[randint], cover_output_path.clone()) {
            Ok(_) => {
                if let Ok(_) = display_full_image_with_chafa(cover_output_path) {
                    // Successfully displayed the image
                } else {
                    eprintln!("Failed to display image.");
                }
            }
            Err(e) => eprintln!("Failed to extract album cover: {}", e),
        }
    } else {
        // Handle case where the file does not contain flac_checker
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

fn play_previous_track(played_songs: Vec<usize>, music_list: Vec<String>, sink: &Sink) {
    if played_songs.len() >= 2 {
        let previous_index = played_songs[played_songs.len() - 2];
        println!("Playing previous song: {}", music_list[previous_index]);
        let file = BufReader::new(File::open(&music_list[previous_index]).unwrap());
        let source = Decoder::new(file).unwrap();
        sink.stop();
        sink.append(source);
        sink.play();
    } else {
        println!("No previous song to play or it's the first song.");
    }
}
*/
