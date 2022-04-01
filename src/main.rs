use std::{
    io,
    io::{Read, Write},
    time::Duration,
};

use survey::sync_tcp::{SyncTcpListener, SyncTcpStream};

const PORT: u16 = 6547;

pub fn main() {
    match listener() {
        Ok(()) => {}
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    }
}

pub fn listener() -> io::Result<()> {
    let mut threads = Vec::new();

    let listener = SyncTcpListener::bind_any(PORT)?;

    println!("Bound listener on port {PORT}");

    for stream in listener.incoming() {
        let stream = stream?;
        let handle = std::thread::spawn(move || handler_thread(stream));
        threads.push(handle);
    }

    for thread in threads {
        thread.join().unwrap();
    }

    Ok(())
}

fn handler_thread(stream: SyncTcpStream) {
    match handler(stream) {
        Ok(()) => {}
        Err(err) => eprintln!("An error occurred while processing connection: {err}"),
    }
}

fn handler(mut stream: SyncTcpStream) -> io::Result<()> {
    println!("Received connection! {stream:?}");

    stream.write_all(b"Hi! Write your favourite three characters: ")?;
    let mut buf = [0u8; 3];
    stream.read_exact(&mut buf)?;
    println!("Read data: {buf:?}");
    stream.write_all(b"\nAh, it's: '")?;
    stream.write_all(&buf)?;
    stream.write_all(b"'. I like them too owo")?;
    std::thread::sleep(Duration::from_millis(100));
    Ok(())
}
