use std::fs::File;
use std::io::BufReader;
use rodio::{Decoder, OutputStream, Sink};

fn main() {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let file = BufReader::new(File::open("/home/jernej/RustyPlayer/src/189 - Amy MacDonald - This Is The Life.flac").unwrap());
    let sink = Sink::try_new(&stream_handle).unwrap();

// Add a dummy source of the sake of the example.
    let source = Decoder::new(file).unwrap();
    sink.append(source);

// The sound plays in a separate thread. This call will block the current thread until the sink
// has finished playing all its queued sounds.
    sink.sleep_until_end();
}