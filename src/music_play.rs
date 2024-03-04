use std::{fs::File, fs, io::BufReader, time::Duration, process::Command, sync::{/*Arc, Mutex,*/atomic::{AtomicBool, Ordering}}, thread, env};
use rand::{Rng};
use rodio::{Decoder, OutputStream, Sink};
use audiotags::{Tag};
use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, PlatformConfig};


static IS_PAUSED: AtomicBool = AtomicBool::new(false);
static LAST_PAUSED_STATE: AtomicBool = AtomicBool::new(false); // Start in a play state
static SHOULD_SKIP: AtomicBool = AtomicBool::new(false);
static SHOULD_PLAY_PREVIOUS: AtomicBool = AtomicBool::new(false);
const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");

fn convert_to_duration(option_seconds: Option<f64>) -> Option<Duration> {
    option_seconds.map(|secs| Duration::from_secs_f64(secs))
}

fn randomizer(music_list: &[String]) -> usize {
    let mut rng = rand::thread_rng();
    return rng.gen_range(0..music_list.len());;

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
        let song_index = randomizer(music_list);
        if debug_mode{println!("Number genereated: {}", song_index)}
    song_history[0].push(song_index);  // Track played songs
    music_player(music_list, debug_mode,song_history, song_index/*&mut rng*/);
}
    }

       // while !sink.empty() && !SHOULD_SKIP.load(Ordering::SeqCst) && !SHOULD_PLAY_PREVIOUS.load(Ordering::SeqCst) {


fn music_player(music_list: &[String], debug_mode: bool, song_history: &mut Vec<Vec<usize>>, song_index: usize) {
        if debug_mode { println!("Playing song number: {}", song_index) }
        if debug_mode { println!("Song numbers played: {:?}", song_history) }
        if debug_mode { println!("Playing song file: {}", music_list[song_index]); }
        // println!("Played songs index: {:?}", song_history);
        // Get an output stream handle to the default physical sound device
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let file = BufReader::new(File::open(&music_list[song_index]).unwrap());
        let mut sink = Sink::try_new(&stream_handle).unwrap();

        let tag = Tag::new().read_from_path(&music_list[song_index]).unwrap();
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
        // sink.set_speed(40.0);


        // Construct the path to the directory where the .jpg files are located


        let cover_output_path = format!("/tmp/{}-cover-{}.jpg", PACKAGE_NAME, song_index);
        let cover_output_path_clone = cover_output_path.clone();


        if debug_mode { println!("Cover export path is: {}", cover_output_path) }
        terminal_ui(&music_list, song_index, title, album, artists.clone(), debug_mode, cover_output_path_clone);

        if song_history[0].len() >= 2 {
            let path_to_file = format!("/tmp/{}-cover-{}.jpg", PACKAGE_NAME, song_history[0][song_history[0].len() - 2]); // Hahahaha this shit si going to be so fucking confusing in like 2 days
            println!("{path_to_file}");

            // Attempt to delete the file
            fs::remove_file(path_to_file);
        }


        #[cfg(not(target_os = "windows"))]
            let hwnd = None;

        #[cfg(target_os = "windows")]
            let hwnd = {
            use raw_window_handle::WindowHandle;
            let handle: WindowsHandle = unimplemented!();
            Some(handle.hwnd)
        };

        let config = PlatformConfig {
            dbus_name: PACKAGE_NAME,
            display_name: "Rusty Player",
            hwnd,
        };

        let mut controls = MediaControls::new(config).unwrap();

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
                        SHOULD_PLAY_PREVIOUS.store(true, Ordering::SeqCst);
                    }

                    // MediaControlEvent::SetVolume(volume) => {
                    //     if debug_mode { println!("{:?} event received via MPRIS",event)}
                    //     sink.set_volume(volume as f32);
                    //     if debug_mode { println!("Volume should be set to {}",volume)}
                    // }


                    // Add more event handlers as needed
                    _ => println!("Event received: {:?}. If you see this message contact me I probably just haven't added support yet for it", event),
                })
            .unwrap();

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


        //    println!("TEST, {album}, {title}, {:?}", artists);


        while !sink.empty() && !SHOULD_SKIP.load(Ordering::SeqCst) && !SHOULD_PLAY_PREVIOUS.load(Ordering::SeqCst) {

            // Check and handle play/pause state...
            let current_paused_state = IS_PAUSED.load(Ordering::SeqCst);
            if current_paused_state != LAST_PAUSED_STATE.load(Ordering::SeqCst) {
                if current_paused_state {
                    if debug_mode { println!("Attempting to pause") }
                    sink.pause();
                    if debug_mode { println!("Track should be paused") }
                    // println!("Paused");
                } else if !current_paused_state {
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


        while !sink.empty() {
            if SHOULD_SKIP.load(Ordering::SeqCst) {
                SHOULD_SKIP.store(false, Ordering::SeqCst);  // Reset the flag
                if debug_mode { println!("Attempting to skip to the next track..."); }
                // std::mem::drop(sink);
                sink.stop();
            }

            if SHOULD_PLAY_PREVIOUS.load(Ordering::SeqCst) {
                SHOULD_PLAY_PREVIOUS.store(false, Ordering::SeqCst);  // Reset the flag
                if debug_mode { println!("Attempting to go back to the previous track..."); }

                // Logic to play the previous track, adjust `current_index` as needed
                if song_history[0].len() >= 2 {
                    sink.stop();
                    song_history[1].push(song_index);
                    let song_index = song_history[0][song_history[0].len() - 2];
                    song_history[0].pop();
                    return music_player(music_list, debug_mode, song_history, song_index/*&mut rng*/);
                } else {
                    println!("Not enough songs in the play queue")
                }
            }

            // Continue existing play/pause state checks here...
            thread::sleep(Duration::from_millis(50));
        }


        sink.sleep_until_end();

        // if song_history[1].len() >= 1 {
        //     let song_index = song_history[1][song_history[1].len() - 1];
        //     song_history[1].pop();
        //     music_player(music_list, debug_mode, song_history, song_index/*&mut rng*/);
        // } else {
        //     random_passer(music_list, debug_mode, song_history);
        // }
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

fn terminal_ui(music_list: &[String], song_index: usize, title: &str, artists: &str, album: String, debug_mode: bool, cover_output_path: String) {
    if !debug_mode { clearscreen::clear().expect("Failed to clear screen"); }

    let flac_checker = ".flac";

    if music_list[song_index].contains(flac_checker) {
        match extract_cover_from_flac(&music_list[song_index], cover_output_path.clone()) {
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

