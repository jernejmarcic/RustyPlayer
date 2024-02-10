use std::fs::File;
use std::{io, thread};
use std::io::BufReader;
use anyhow::Result;
use rand::Rng;
use rodio::{Decoder, OutputStream, Sink};
use std::path::PathBuf;
use std::time::Duration;
use audiotags::Tag;
use termion::event::Key;


pub(crate) fn play_random_song(music_list: &[String]) -> std::io::Result<()> {
    if music_list.is_empty() {
        println!("No songs found in the specified directory.");
        return Ok(());
    }

    let mut rng = rand::thread_rng();

  //  loop {
        let randint = rng.gen_range(0..music_list.len());
        println!("Playing song: {}", music_list[randint]);
        // Get a output stream handle to the default physical sound device
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let file = BufReader::new(File::open(&music_list[randint]).unwrap());
        let sink = Sink::try_new(&stream_handle).unwrap();

// Add a dummy source of the sake of the example.
        let source = Decoder::new(file).unwrap();
        let tag = Tag::new().read_from_path(&music_list[randint]).unwrap();
        let title = tag.title().unwrap_or_else(|| "Unknown".into());
        let artists = tag
            .artists()
            .map(|a| a.join(", "))
            .unwrap_or_else(|| "Unknown".into());
        let album = tag.album_title().unwrap_or_else(|| "Unknown".into());

        println!("Title:     {} \r", title);
        println!("Artists:   {} \r", artists);
        println!("Album:     {} \r", album);
       // sink.append(source);

// The sound plays in a separate thread. This call will block the current thread until the sink
// has finished playing all its queued sounds.
       // sink.sleep_until_end();

  //  }

    Ok(())
}
