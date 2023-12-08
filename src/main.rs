mod lorentz;
use lorentz::*;

mod stdin;

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 1800;
const HEIGHT: usize = 1200;

const MIN_T: f64 = 0.0;
const MAX_T: f64 = 10.0;
const STEP_T: f64 = 0.001;
const C: f64 = 100.0;

#[derive(Serialize, Deserialize)]
struct Config {
    object: Vec<Object>,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
struct Object {
    lifetime: [f64; 2], // given in `main-frame` time.
    color: String,
    velocity: [f64; 2],
    xy_offset: [f64; 2], // The position of `self` during the big bang.
}

impl Object {
    fn frame(&self) -> Frame {
        Frame {
            velocity: self.velocity,
        }
    }
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
    let stdin = stdin::mk_channel();

    let mut frame = Frame::main();

    // `main-frame` time.
    let mut t = MIN_T;

    while window.is_open() && !window.is_key_down(Key::Escape) && t < MAX_T {
        buffer.iter_mut().for_each(|x| *x = 0);

        if let Ok(line) = stdin.try_recv() {
            if line.starts_with("set-frame ") {
                let arg = line["set-frame ".len() .. ].trim();
                if arg == "main" {
                    frame = Frame::main();
                    println!("frame set to main frame");
                } else {
                    let i = arg.parse::<usize>().unwrap();
                    frame = config.object[i].frame();
                    println!("frame set to object {}", i);
                }
            }
        }

        for obj in &config.object {
            if obj.lifetime[0] > t || obj.lifetime[1] < t { continue; }
            let x = obj.xy_offset[0] + obj.velocity[0] * t;
            let y = obj.xy_offset[1] + obj.velocity[1] * t;
            let c = get_color(&obj.color);

            let ev = Event {
                t,
                xy: [x, y],
            };
            let Event { xy: [x, y], t: t2 } = frame.from_other_frame(Frame::main(), ev, Some(C));

            const R: i32 = 5;
            for x_ in -R..=R {
                for y_ in -R..=R {
                    let ev = Event {
                        t: t2,
                        xy: [x + x_ as f64, y + y_ as f64],
                    };
                    let ev = frame.from_other_frame(obj.frame(), ev, Some(C));
                    if ev.xy[0] < 0.0 || ev.xy[0] > WIDTH as f64 { continue; }
                    if ev.xy[1] < 0.0 || ev.xy[1] > HEIGHT as f64 { continue; }
                    buffer[ev.xy[0] as usize + ev.xy[1] as usize * WIDTH] = c;
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
