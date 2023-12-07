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

const MIN_T: f64 = 0.0;
const MAX_T: f64 = 1.0;
const STEP_T: f64 = 0.01;

fn get_color(s: &str) -> u32 {
    match s {
        "red" => 0xff0000,
        "blue" => 0x0000ff,
        "green" => 0x00ff00,
        _ => panic!(),
    }
}

fn main() {
    let mut f = File::open("input.toml").unwrap();
    let mut data = String::new();
    f.read_to_string(&mut data).unwrap();
    let config: Config = toml::from_str(&*data).unwrap();

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

    let mut t = MIN_T;
    while window.is_open() && !window.is_key_down(Key::Escape) && t < MAX_T {
        buffer.iter_mut().for_each(|x| *x = 0);

        for obj in &config.object {
            if obj.lifetime[0] > t || obj.lifetime[1] < t { continue; }
            let x = obj.xy_offset[0] + obj.velocity[0] * t;
            let y = obj.xy_offset[1] + obj.velocity[1] * t;

            let c = get_color(&obj.color);
            for x_ in [-2.0, -1.0, 0.0, 1.0, 2.0] {
                for y_ in [-2.0, -1.0, 0.0, 1.0, 2.0] {
                    let x = x + x_;
                    let y = y + y_;
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
