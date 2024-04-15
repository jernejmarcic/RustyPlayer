use std::{fs::File,
          fs, io::BufReader,
          time::Duration,
          process::Command,
          sync::{/*Arc, Mutex,*/
                 atomic::{AtomicBool, Ordering}},
          thread,
          env};
use rand::{Rng};
use rodio::{Decoder, OutputStream, Sink};
use audiotags::{Tag};
use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, PlatformConfig};
use async_recursion::async_recursion;

static IS_PAUSED: AtomicBool = AtomicBool::new(false);
static LAST_PAUSED_STATE: AtomicBool = AtomicBool::new(false);
// Start in a play state
static SHOULD_SKIP: AtomicBool = AtomicBool::new(false);
static SHOULD_PLAY_PREVIOUS: AtomicBool = AtomicBool::new(false);
const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");

fn convert_to_duration(option_seconds: Option<f64>) -> Option<Duration> {
    option_seconds.map(|secs| Duration::from_secs_f64(secs))
}

pub(crate) fn play_random_song(music_list: &[String], debug_mode: bool /*, config_path: */) -> std::io::Result<()> {
    if music_list.is_empty() {
        println!("No songs found in the specified directory.");
        return Ok(());
    }
    let mut song_history: Vec<Vec<usize>> = vec![vec![], vec![]]; // Initialize with two empty rows
    // println!("Song history {:?}", song_history);
    random_passer(music_list, debug_mode, &mut song_history);
    Ok(())
}

fn random_passer(music_list: &[String], debug_mode: bool, song_history: &mut Vec<Vec<usize>>) {

//    let mut last_paused_state = IS_PAUSED.load(Ordering::SeqCst);
    //       let current_paused_state = IS_PAUSED.load(Ordering::SeqCst);
loop {
    let mut rng = rand::thread_rng();
    let song_index = rng.gen_range(0..music_list.len());
    if debug_mode { println!("Number genereated: {}", song_index) }
    song_history[0].push(song_index);  // Track played songs
    println!("Song numbers played after finish: {:?}", song_history);
    music_player(music_list, debug_mode, song_history, song_index/*&mut rng*/);

}
}

// while !sink.empty() && !SHOULD_SKIP.load(Ordering::SeqCst) && !SHOULD_PLAY_PREVIOUS.load(Ordering::SeqCst) {

fn music_player(music_list: &[String], debug_mode: bool, song_history: &mut Vec<Vec<usize>>, song_index: usize) {
    if debug_mode { println!("Playing song number: {}", song_index) }
    if debug_mode { println!("Song numbers played: {:?}", song_history) }
    if debug_mode { println!("Playing song file: {}", music_list[song_index]); }
    // println!("Played songs index: {:?}", song_history);
    // Get an output stream handle to the default physical sound device

// Attempt to acquire the default audio output stream of the system.
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
// Open the audio file from the list at the given index and wrap it in a buffered reader.
    let file = BufReader::new(File::open(&music_list[song_index]).unwrap());
// Create a new audio sink attached to the audio output stream for playing sound.
    let sink = Sink::try_new(&stream_handle).unwrap();
// Decode the audio file for playback.
    let source = Decoder::new(file).unwrap();
// Append the decoded audio source to the sink for playback.
    sink.append(source);


// Create a new tag reader object to read metadata from the audio file
    let tag = Tag::new().read_from_path(&music_list[song_index]).unwrap();
// Extract the title from the audio file's metadata, defaulting to "Unknown" if not present
    let title = tag.title().unwrap_or_else(|| "Unknown".into());
// Attempt to extract the duration of the audio file in seconds, if available
    let duration_seconds: Option<f64> = tag.duration();  // Duration is optionally returned in seconds
// Convert the duration from seconds (f64) to a Duration object, if duration_seconds is Some
    let duration: Option<Duration> = convert_to_duration(duration_seconds);
// Extract the artist(s) from the audio file's metadata, joining multiple artists with a comma if necessary
    let artists = tag
        .artists()
        .map(|a| a.join(", "))
        .unwrap_or_else(|| "Unknown".into());
// Extract the album title from the audio file's metadata, defaulting to "Unknown" if not present
    let album = tag.album_title().unwrap_or_else(|| "Unknown".into());

// Construct the path to the directory where the .jpg files are located
    let cover_output_path = format!("/tmp/{}-cover-{}.jpg", PACKAGE_NAME, song_index);

    if debug_mode { println!("Cover export path is: {}", cover_output_path) }
    terminal_ui(&music_list, song_index, title, album, artists.clone(), debug_mode, cover_output_path.clone());

    if song_history[0].len() >= 2 {
        let path_to_file = format!("/tmp/{}-cover-{}.jpg", PACKAGE_NAME, song_history[0][song_history[0].len() - 2]); // Hahahaha this shit si going to be so fucking confusing in like 2 days
       if debug_mode { println!("{path_to_file}");}

        // Attempt to delete the file
        fs::remove_file(path_to_file);
    }


    // Conditional compilation for non-Windows platforms
    #[cfg(not(target_os = "windows"))]
        let hwnd = None;

// Conditional compilation for Windows platforms
    #[cfg(target_os = "windows")]
        let hwnd = {
        use raw_window_handle::WindowHandle;
        // This part should be implemented to retrieve a valid window handle for Windows platforms
        let handle: WindowsHandle = unimplemented!();
        Some(handle.hwnd)
    };

// Configuration for media controls
    let config = PlatformConfig {
        dbus_name: PACKAGE_NAME, // Unique D-Bus name for the application
        display_name: "Rusty Player", // Display name of the player
        hwnd, // Window handle, relevant on Windows for global media controls
    };

// Initialize media controls with the provided configuration

// Initialize media controls with the provided configuration
    let mut controls = MediaControls::new(config).unwrap();

// Attach event listeners for media control events
    controls
        .attach(move |event: MediaControlEvent| match event {
            MediaControlEvent::Play => {
                // Event handler for play command
                if debug_mode { println!("{:?} event received via MPRIS", event) }
                let current_state = IS_PAUSED.load(Ordering::SeqCst);
                IS_PAUSED.store(!current_state, Ordering::SeqCst);  // Toggle play/pause state
            },
            MediaControlEvent::Pause => {
                // Event handler for pause command
                if debug_mode { println!("{:?} event received via MPRIS", event) }
                let current_state = IS_PAUSED.load(Ordering::SeqCst);
                IS_PAUSED.store(!current_state, Ordering::SeqCst);  // Toggle play/pause state
            },
            MediaControlEvent::Toggle => {
                // Event handler for toggle play/pause command
                if debug_mode { println!("{:?} event received via MPRIS", event) }
                let current_state = IS_PAUSED.load(Ordering::SeqCst);
                IS_PAUSED.store(!current_state, Ordering::SeqCst);  // Toggle play/pause state
            },
            MediaControlEvent::Next => {
                // Event handler for skip to next track command
                if debug_mode { println!("{:?} event received via MPRIS", event) }
                SHOULD_SKIP.store(true, Ordering::SeqCst); // Signal to skip to the next track
            },
            MediaControlEvent::Previous => {
                // Event handler for skip to previous track command
                if debug_mode { println!("{:?} event received via MPRIS", event) }
                SHOULD_PLAY_PREVIOUS.store(true, Ordering::SeqCst); // Signal to play the previous track
            },
            // Additional event handlers can be implemented here
            _ => println!("Event received: {:?}. If you see this message contact me I probably just haven't added support yet for it", event),
        })
        .unwrap();

// Update media metadata for the current playing track
    controls
        .set_metadata(MediaMetadata {
            title: Some(title), // Track title
            artist: Some(&*artists), // Artist name(s)
            album: Some(album), // Album name
            duration: duration, // Track duration
            cover_url: Some(&*format!("file://{}", cover_output_path)), // URL to the album cover image
            ..Default::default()
        })
        .unwrap();





    // Loop continues as long as the audio sink is not empty, and neither skip nor previous track events have been triggered
    while !sink.empty() && !SHOULD_SKIP.load(Ordering::SeqCst) && !SHOULD_PLAY_PREVIOUS.load(Ordering::SeqCst) {

        // Checks the current paused state against the last known paused state
        let current_paused_state = IS_PAUSED.load(Ordering::SeqCst);
        if current_paused_state != LAST_PAUSED_STATE.load(Ordering::SeqCst) {
            // If the track is currently paused, attempt to pause playback
            if current_paused_state {
                if debug_mode { println!("Attempting to pause") }
                sink.pause(); // Pauses the playback
                if debug_mode { println!("Track should be paused") }
            } else if !current_paused_state { // If the track is currently playing, attempt to resume playback
                if debug_mode { println!("Attempting to resume/play") }
                sink.play(); // Resumes the playback
                if debug_mode { println!("Track should be resumed") }
            }
            // Updates the last paused state to reflect the current state
            LAST_PAUSED_STATE.store(current_paused_state, Ordering::SeqCst);
        }

        // Sleep briefly to reduce CPU usage and allow time for playback state changes or commands
        thread::sleep(Duration::from_millis(50));
    }



    // Loop continues as long as there are sounds queued in the sink.
    while !sink.empty() {
        // Check if there's a request to skip to the next track.
        if SHOULD_SKIP.load(Ordering::SeqCst) {
            // Reset the skip flag.
            SHOULD_SKIP.store(false, Ordering::SeqCst);
            if debug_mode { println!("Attempting to skip to the next track..."); }
            // Stop playback, effectively skipping the current track.
            sink.stop();
        }

        // Check if there's a request to play the previous track.
        if SHOULD_PLAY_PREVIOUS.load(Ordering::SeqCst) {
            // Reset the play previous flag.
            SHOULD_PLAY_PREVIOUS.store(false, Ordering::SeqCst);
            if debug_mode { println!("Attempting to go back to the previous track..."); }

            // Logic to play the previous track if there are enough entries in the song history.
            if song_history[0].len() >= 2 {
                // Stop the current track.
                sink.stop();
                // Prepare to play the previous track by adjusting the song index.
                song_history[1].push(song_index);
                let song_index = song_history[0][song_history[0].len() - 2];
                // Remove the last entry in the current song history.
                song_history[0].pop();
                // Play the previous track.
                music_player(music_list, debug_mode, song_history, song_index);
            } else {
                // If there are not enough songs in the history, notify the user.
                println!("Not enough songs in the play queue");
            }
        }

        // Perform routine checks, such as play/pause state, every 50 milliseconds.
        thread::sleep(Duration::from_millis(50));
    }



    sink.sleep_until_end();

    if song_history[1].len() >= 1 {
        let song_index = song_history[1][song_history[1].len() - 1];
        song_history[1].pop();
        music_player(music_list, debug_mode, song_history, song_index);
    }
}


// This function attempts to extract the cover image from a FLAC file.
// It takes the path to the FLAC file and the desired output path for the cover image as arguments.
fn extract_cover_from_flac(flac_path: &str, cover_output_path: String) -> Result<(), std::io::Error> {
    // Execute the `metaflac` command to export the picture (album cover) to the specified output path.
    let output = Command::new("metaflac")
        .arg("--export-picture-to")
        .arg(&cover_output_path)
        .arg(flac_path)
        .output();

    // Handle the command's output to check for success or failure.
    match output {
        Ok(output) if output.status.success() => Ok(()), // If the command was successful, return Ok.
        Ok(output) => {
            // If the command executed but failed, print the error message and return an error.
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("metaflac command failed: {}", stderr);
            Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to extract album cover"))
        },
        Err(e) => Err(e), // If executing the command itself failed, return the encountered error.
    }
}


// This function uses the `chafa` command-line tool to display an image as ASCII art in the terminal.
// It takes the path to the image as an argument.
fn display_full_image_with_chafa(image_path: String) -> Result<(), std::io::Error> {
    // Execute the `chafa` command with specified dimensions and the path to the image.
    let output = Command::new("chafa")
        .arg("-s")
        .arg("70x25") // The size of the output in columns and rows. TODO: Make configurable.
        .arg(image_path)
        .output()?;

    // Check if the command was successfully executed and print the ASCII art to the terminal.
    if output.status.success() {
        print!("{} \r", String::from_utf8_lossy(&output.stdout)); // Print the ASCII art.
        Ok(()) // Return Ok if successful.
    } else {
        // If the command failed, return an error with a message indicating the failure.
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to display image",
        ))
    }
}


// This function updates the terminal UI with the current song's metadata and potentially its album cover.
fn terminal_ui(music_list: &[String], song_index: usize, title: &str, artists: &str, album: String, debug_mode: bool, cover_output_path: String) {
    // Clears the terminal screen unless debug mode is active, to keep the terminal output clean.
    if !debug_mode { clearscreen::clear().expect("Failed to clear screen"); }

    // Check if the current song file is a FLAC file to attempt extracting and displaying its cover.
    let flac_checker = ".flac";
    if music_list[song_index].contains(flac_checker) {
        // Attempt to extract the cover from the FLAC file and display it if successful.
        match extract_cover_from_flac(&music_list[song_index], cover_output_path.clone()) {
            Ok(_) => {
                // Attempt to display the image using `chafa` if the cover was successfully extracted.
                if let Ok(_) = display_full_image_with_chafa(cover_output_path) {
                    // Successfully displayed the image in the terminal.
                } else {
                    eprintln!("Failed to display image."); // Error handling if the image could not be displayed.
                }
            },
            Err(e) => eprintln!("Failed to extract album cover: {}", e), // Error handling if the cover could not be extracted.
        }
    } else {
        // Additional handling could go here for non-FLAC files or if the FLAC file does not contain an image.
    }

    // Print song metadata to the terminal.
    println!("Title:     {} \r", title); // Displays the song title.
    println!("Artists:   {} \r", artists); // Displays the song artists.
    println!("Album:     {} \r", album); // Displays the album name.
    // Duration could also be displayed here if needed.
}


