mod lorentz;
use lorentz::*;

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 1800;
const HEIGHT: usize = 1200;

const STEP_T: f64 = 0.01;

#[derive(Serialize, Deserialize)]
struct Config {
    c: f64,
    observer_velocity: [f64; 2],
    object: Vec<Object>,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
struct Object {
    start: Event,
    end: Event,
    color: String,
}

fn get_color(s: &str) -> u32 {
    match s {
        "red" => 0xff0000,
        "blue" => 0x0000ff,
        "green" => 0x00ff00,
        "yellow" => 0xffff00,
        "violet" => 0xff00ff,
        _ => panic!(),
    }
}

fn load_config() -> Config {
    let mut f = File::open("input.toml").unwrap();
    let mut data = String::new();
    f.read_to_string(&mut data).unwrap();

    toml::from_str(&*data).unwrap()
}

fn main() {
    let config = load_config();

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

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    // time within the observers frame.
    let mut t = 0.0;
    let observer_frame = Frame { velocity: config.observer_velocity };

    while window.is_open() && !window.is_key_down(Key::Escape) {
        buffer.iter_mut().for_each(|x| *x = 0);

        for obj in &config.object {
            let start = observer_frame.from_other_frame(Frame::main(), obj.start, Some(config.c));
            let end = observer_frame.from_other_frame(Frame::main(), obj.end, Some(config.c));

            if start.t > t || end.t < t { continue; }

            // d = 0, t = start.t
            // d = 1, t = end.t
            let d = (t - start.t) / (end.t - start.t);
            let x = (1.0 - d) * start.xy[0] + d * end.xy[0];
            let y = (1.0 - d) * start.xy[1] + d * end.xy[1];
            let c = get_color(&obj.color);

            const R: i32 = 5;
            for x_ in -R..=R {
                for y_ in -R..=R {
                    let x = x + x_ as f64;
                    let y = y + y_ as f64;
                    if x < 0.0 || x > WIDTH as f64 { continue; }
                    if y < 0.0 || y > HEIGHT as f64 { continue; }
                    buffer[x as usize + y as usize * WIDTH] = c;
                }
            }
        }

        t += STEP_T;

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }

}
