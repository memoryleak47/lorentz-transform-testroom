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
    object: Vec<Object>,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
struct Object {
    follow: Option<String>,
    color: String,
    path: Vec<Event>,
}

fn get_color(s: &str) -> u32 {
    match s {
        "red" => 0xff0000,
        "blue" => 0x0000ff,
        "green" => 0x00ff00,
        "yellow" => 0xffff00,
        "violet" => 0xff00ff,
        "white" => 0xffffff,
        _ => panic!(),
    }
}

fn load_config() -> Config {
    let mut f = File::open("input.toml").unwrap();
    let mut data = String::new();
    f.read_to_string(&mut data).unwrap();

    toml::from_str(&*data).unwrap()
}

fn frame_of_stage(object: &Object, stage: usize) -> Frame {
    let (start, end) = (object.path[stage], object.path[stage+1]);
    let vx = (start[X] - end[X]) / (end[T] - start[T]);
    let vy = (start[Y] - end[Y]) / (end[T] - start[T]);
    Frame { velocity: [vx, vy] }
}

fn find_stage(object: &Object, observer_frame: Frame, observer_t: f64, config: &Config) -> Option<(usize, Event, Event)> {
    let evs: Vec<Event> = object.path.iter().map(|ev| observer_frame.from_other_frame(Frame::main(), *ev, Some(config.c))).collect();
    for i in 0..evs.len()-1 {
        if evs[i][T] <= observer_t && observer_t < evs[i+1][T] {
            return Some((i, evs[i], evs[i+1]));
        }
    }
    return None;
}

fn main() {
    let config = load_config();
    assert_eq!(config.object.iter().filter(|x| x.follow.is_some()).count(), 1);
    let follow_idx = config.object.iter().position(|x| x.follow.is_some()).unwrap();

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

    let mut stage = 0;
    let mut observer_frame = frame_of_stage(&config.object[follow_idx], stage);

    // time within the observers frame.
    let mut t = 0.0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        buffer.iter_mut().for_each(|x| *x = 0);

        // consider switching stages.
        while observer_frame.from_other_frame(Frame::main(), config.object[follow_idx].path[stage+1], Some(config.c))[T] < t {
            stage += 1;
            if config.object[follow_idx].path.get(stage+1).is_none() {
                panic!("*graceful shutdown*");
            }
            observer_frame = frame_of_stage(&config.object[follow_idx], stage);
            // set time `t` to the starting point of the new stage.
            t = observer_frame.from_other_frame(Frame::main(), config.object[follow_idx].path[stage], Some(config.c))[T];
        }

        // render objects.
        for obj in &config.object {
            let Some((_, start, end)) = find_stage(obj, observer_frame, t, &config) else { continue; };

            // d = 0, t = start.t
            // d = 1, t = end.t
            let d = (t - start[T]) / (end[T] - start[T]);
            let x = (1.0 - d) * start[X] + d * end[X];
            let y = (1.0 - d) * start[Y] + d * end[Y];
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
