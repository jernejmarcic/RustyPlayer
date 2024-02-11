use std::{fs::File,
          io::BufReader,
          path::PathBuf,
          time::Duration,
          sync::mpsc::Sender,
          path::Path,
          process::Command,
          sync::{Arc,
                 atomic::{AtomicBool, Ordering}},
          thread
};
use rand::Rng;
use rodio::{Decoder, OutputStream, Sink};
use audiotags::Tag;
use termion::event::{Key, parse_event};
use dbus::{blocking::Connection, channel::MatchingReceiver, message::MatchRule};
use dbus::blocking::stdintf::org_freedesktop_dbus::EmitsChangedSignal::False;
use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, PlatformConfig};

pub(crate) fn play_random_song(music_list: &[String]) -> std::io::Result<()> {
    if music_list.is_empty() {
        println!("No songs found in the specified directory.");
        return Ok(());
    }

    let mut rng = rand::thread_rng();
    let mut played_songs: Vec<i16> = Vec::new();

    let is_paused = Arc::new(AtomicBool::new(true)); // Initially paused
    let is_paused_clone = is_paused.clone();

    loop {


        // Your actual logic goes here.

        let randint = rng.gen_range(0..music_list.len());
        played_songs.push(randint as i16);
        //println!("Playing song: {}", music_list[randint]);
        // Get a output stream handle to the default physical sound device
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let file = BufReader::new(File::open(&music_list[randint]).unwrap());
        let sink = Sink::try_new(&stream_handle).unwrap();

// Add a dummy source of the sake of the example.
        let source = Decoder::new(file).unwrap();
        let tag = Tag::new().read_from_path(&music_list[randint]).unwrap();
        let title = tag.title().unwrap_or_else(|| "Unknown".into());

        let duration_option: Option<f64> = tag.duration();

// Convert Option<f64> to Option<Duration>
        let duration: Option<Duration> = duration_option.map(|secs| Duration::from_secs_f64(secs));



        let artists = tag
            .artists()
            .map(|a| a.join(", "))
            .unwrap_or_else(|| "Unknown".into());
        let album = tag.album_title().unwrap_or_else(|| "Unknown".into());

        println!("Title:     {} \r", title);
        println!("Artists:   {} \r", artists);
        println!("Album:     {} \r", album);
        println!("Duration: {:?} \r", duration);

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
            .attach(move |event: MediaControlEvent| match event {
                MediaControlEvent::Play => {
                    println!("Play event received");
                    // Logic to play the music
                    is_paused_clone.store(false, Ordering::SeqCst); // Set to not paused


                },
                MediaControlEvent::Pause => {
                    println!("Pause event received");
                    // Logic to pause the music

                },
                MediaControlEvent::Next => {
                    println!("Next event received");
                    // Logic to skip to the next track
                    // You might need to signal your playback loop to move to the next song
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



        sink.append(source);
// The sound plays in a separate thread. This call will block the current thread until the sink
// has finished playing all its queued sounds.
        sink.sleep_until_end();



    }

    Ok(())
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
