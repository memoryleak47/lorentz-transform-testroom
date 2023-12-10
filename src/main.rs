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
#[derive(Debug, Clone)]
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
    let filename = std::env::args().nth(1).unwrap_or_else(|| String::from("input.toml"));
    let mut f = File::open(&filename).unwrap();
    let mut data = String::new();
    f.read_to_string(&mut data).unwrap();

    toml::from_str(&*data).unwrap()
}

struct Ctxt {
    config: Config,
    follow_obj: Object,
    stage: usize,
    t: f64,
    observer_frame: Frame,
    buffer: Vec<u32>,
    window: Window,
}

impl Ctxt {
    fn new() -> Ctxt {
        let config = load_config();
        assert_eq!(config.object.iter().filter(|x| x.follow.is_some()).count(), 1);
        let follow_idx = config.object.iter().position(|x| x.follow.is_some()).unwrap();

        for obj in &config.object {
            for stage in 0..obj.path.len()-1 {
                let v = Self::calc_frame(obj, stage).velocity;
                let len = (v[0] * v[0] + v[1] * v[1]).sqrt();
                assert!(len < config.c, "You cannot move faster than c!");
            }
        }

        let buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

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

        let mut ctxt = Ctxt {
            follow_obj: config.object[follow_idx].clone(),
            config,
            stage: 0,
            t: 0.0, // will be set correctly in set_stage.
            observer_frame: Frame::main(), // will be set correctly in set_stage.
            buffer,
            window
        };

        ctxt.set_stage(0);
        ctxt

    }

    fn run(&mut self) {
        while self.window.is_open() && !self.window.is_key_down(Key::Escape) {
            self.buffer.iter_mut().for_each(|x| *x = 0);

            // consider switching stages.
            while self.main_to_observer(self.follow_obj.path[self.stage+1])[T] < self.t {
                if self.follow_obj.path.get(self.stage+2).is_none() {
                    return;
                }
                self.set_stage(self.stage + 1);
            }

            // This is the camera center point.
            let followed_pixels = self.raw_pixels(&self.follow_obj).unwrap();
            let focus_x = followed_pixels.iter().map(|[x, _]| x).sum::<f64>() / followed_pixels.len() as f64;
            let focus_y = followed_pixels.iter().map(|[_, y]| y).sum::<f64>() / followed_pixels.len() as f64;

            // render objects.
            for obj in &self.config.object {
                let Some(pixels) = self.raw_pixels(obj) else { continue; };
                let c = get_color(&obj.color);

                for [x, y] in pixels {
                    let x: f64 = x - focus_x + WIDTH as f64/2.0;
                    let y: f64 = y - focus_y + HEIGHT as f64/2.0;
                    if x < 0.0 || x > WIDTH as f64 { continue; }
                    if y < 0.0 || y > HEIGHT as f64 { continue; }
                    self.buffer[x as usize + y as usize * WIDTH] = c;
                }
            }

            self.t += STEP_T;

            // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
            self.window
                .update_with_buffer(&self.buffer, WIDTH, HEIGHT)
                .unwrap();
        }
    }

    fn set_stage(&mut self, stage: usize) {
        self.stage = stage;
        self.observer_frame = Self::calc_frame(&self.follow_obj, stage);
        self.t = self.main_to_observer(self.follow_obj.path[self.stage])[T];
    }

    // translates an event from the `main-frame` (eg. taken from the config file) to the observer-frame.
    fn main_to_observer(&self, ev: Event) -> Event {
        self.observer_frame.from_other_frame(Frame::main(), ev, Some(self.config.c))
    }

    fn find_stage(&self, obj: &Object) -> Option<(usize, Event, Event)> {
        let evs: Vec<Event> = obj.path.iter().map(|ev| self.main_to_observer(*ev)).collect();
        for i in 0..evs.len()-1 {
            if evs[i][T] <= self.t && self.t < evs[i+1][T] {
                return Some((i, evs[i], evs[i+1]));
            }
        }
        return None;
    }

    fn raw_pixels(&self, obj: &Object) -> Option<Vec<[f64; 2]>> {
        let (stage, start, end) = self.find_stage(obj)?;
        let f = Self::calc_frame(obj, stage);

        let mut pixels = Vec::new();

        const R: i32 = 5;
        for x_ in -R..=R {
            for y_ in -R..=R {

                // calculate start & end in objs resting frame f.
                let start = f.from_other_frame(self.observer_frame, start, Some(self.config.c));
                let end = f.from_other_frame(self.observer_frame, end, Some(self.config.c));

                // add offsets there.
                let start = [start[X] + x_ as f64, start[Y] + y_ as f64, start[T]];
                let end = [end[X] + x_ as f64, end[Y] + y_ as f64, end[T]];

                // transform back to the observers frame.
                let start = self.observer_frame.from_other_frame(f, start, Some(self.config.c));
                let end = self.observer_frame.from_other_frame(f, end, Some(self.config.c));

                let d = (self.t - start[T]) / (end[T] - start[T]);
                let x = (1.0 - d) * start[X] + d * end[X];
                let y = (1.0 - d) * start[Y] + d * end[Y];

                pixels.push([x, y]);
            }
        }

        Some(pixels)
    }

    fn calc_frame(obj: &Object, stage: usize) -> Frame {
        let (start, end) = (obj.path[stage], obj.path[stage+1]);
        let vx = (end[X] - start[X]) / (end[T] - start[T]);
        let vy = (end[Y] - start[Y]) / (end[T] - start[T]);
        Frame { velocity: [vx, vy] }
    }
}

fn main() {
    Ctxt::new().run();
}
