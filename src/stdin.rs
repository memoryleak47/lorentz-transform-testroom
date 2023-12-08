use std::sync::mpsc::*;
use std::io::BufRead;

pub fn mk_channel() -> Receiver<String> {
    let (s, r) = channel::<String>();
    std::thread::spawn(move || {
        loop {
            let mut buffer = String::new();
            let stdin = std::io::stdin();
            let mut handle = stdin.lock();

            handle.read_line(&mut buffer).unwrap();
            s.send(buffer).unwrap();
        }
    });

    r
}
