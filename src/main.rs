use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use minifb::{Key, Window, WindowOptions};

#[derive(Serialize, Deserialize)]
struct Config {
    object: Vec<Object>,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
struct Object {
    lifetime: [f64; 2],
    color: String,
    velocity: [f64; 2],
    t_offset: f64,
    xy_offset: [f64; 2],
}

const WIDTH: usize = 640;
const HEIGHT: usize = 360;

fn main() {
    let mut f = File::open("input.toml").unwrap();
    let mut data = String::new();
    f.read_to_string(&mut data).unwrap();
    let config: Config = toml::from_str(&*data).unwrap();

    for obj in config.object {
        dbg!(obj);
    }

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for i in buffer.iter_mut() {
            *i = 255 + (255 << 16) + (255 << 8); // write something more funny here!
        }

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }

}
