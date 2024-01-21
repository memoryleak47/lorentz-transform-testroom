use crate::*;

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;


#[derive(Serialize, Deserialize)]
pub struct Config {
    pub c: f64,
    pub tick_delta: f64,
    pub max_clock: f64,
    pub object: Vec<Object>,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone)]
pub struct Object {
    pub follow: Option<String>,
    pub clock: Option<String>,
    pub color: String,
    pub path: Vec<Event>,
}

impl Config {
    pub fn load() -> Config {
        let filename = std::env::args().nth(1).unwrap_or_else(|| {
            eprintln!("Missing config file argument. You can pick an example from the examples folder");
            eprintln!("Run using `cargo run <config-file>`");
            std::process::exit(1);
        });
        let mut f = File::open(&filename).unwrap();
        let mut data = String::new();
        f.read_to_string(&mut data).unwrap();

        toml::from_str(&*data).unwrap()
    }

    pub fn to_ctxt(self) -> Ctxt {
        // generate pixel_objects:
        let mut pixel_objects = Vec::new();

        let mut follow_obj: Option<PixelObject> = None;

        const R: i32 = 20;
        for obj in &self.object {
            for y in -R..=R {
                for x in -R..=R {
                    let px = mk_pixel_object(obj, x, y, self.c);

                    if x == 0 && y == 0 && obj.follow.is_some() {
                        assert!(follow_obj.is_none());
                        follow_obj = Some(px.clone());
                    }

                    pixel_objects.push(px);
                }
            }
        }

        // faster than c check!
        for pobj in &pixel_objects {
            for stage in 0..pobj.path.len()-1 {
                let v = Ctxt::calc_frame(&pobj.path, stage).velocity;
                let len = (v[0] * v[0] + v[1] * v[1]).sqrt();
                assert!(len < self.c, "You cannot move faster than c!");
            }
        }

        // putting it together.
        let mut ctxt = Ctxt {
            follow_obj: follow_obj.unwrap(),
            pixel_objects,
            tick_delta: self.tick_delta,
            c: self.c,
            stage: 0,
            t: 0.0, // will be set correctly in set_stage.
            observer_frame: Frame::main(), // will be set correctly in set_stage.
            graphics: Graphics::new(),
        };

        // setup initial stage
        ctxt.set_stage(0);

        ctxt
    }
}

// from the rest frame of an object, it is spawned "at once", and it dies "at once".
// within a stage change, the acceleration time delta can be calculated using the required length contraction in either frame.
fn mk_pixel_object(object: &Object, x: i32, y: i32, c: f64) -> PixelObject {
    let color = get_color(&object.color);

    let mut path = Vec::new();

    // start event
    let start_frame = Ctxt::calc_frame(&object.path, 0);
    let start_ev = *object.path.first().unwrap();
    path.push(simultaneous_event(start_frame, start_ev, x, y, c));

    // transition events
    for i in 0..object.path.len() - 2 {
        let f1 = Ctxt::calc_frame(&object.path, i);
        let f2 = Ctxt::calc_frame(&object.path, i+1);
        let ev = object.path[i+1];
        path.push(transition_event(f1, f2, ev, x, y, c));
    }

    // end event
    let end_frame = Ctxt::calc_frame(&object.path, object.path.len() - 2);
    let end_ev = *object.path.last().unwrap();
    path.push(simultaneous_event(end_frame, end_ev, x, y, c));

    PixelObject {
        path,
        color,
    }
}

fn simultaneous_event(f: Frame, ev: Event, x: i32, y: i32, c: f64) -> Event {
    let ev = f.from_other_frame(Frame::main(), ev, Some(c));
    let ev = [ev[X] + x as f64, ev[Y] + y as f64, ev[T]];
    let ev = Frame::main().from_other_frame(f, ev, Some(c));
    ev
}

fn sqr(x: f64) -> f64 { x * x }

fn transition_event(f1: Frame, f2: Frame, ev: Event, x: i32, y: i32, c: f64) -> Event {
    let ev1 = f1.from_other_frame(Frame::main(), ev, Some(c));
    let ev1 = [ev1[X] + x as f64, ev1[Y] + y as f64, ev1[T]];

    let ev2 = f2.from_other_frame(Frame::main(), ev, Some(c));
    let ev2 = [ev2[X] + x as f64, ev2[Y] + y as f64, ev2[T]];

    let mut delta_t = 0.0;

    if f1.velocity != f2.velocity {
        // we want to minimize t_dist.
        let t_dist = |t| {
            let ev1_plus_t = [ev1[X], ev1[Y], ev1[T] + t];
            let ev2_ = f2.from_other_frame(f1, ev1_plus_t, Some(c));

            sqr(ev2_[X] - ev2[X]) + sqr(ev2_[Y] - ev2[Y])
        };

        let mut min = -10000.0;
        let mut max = 10000.0;

        for _ in 0..100 {
            let center = (min + max)/2.0;
            if t_dist(min) < t_dist(max) {
                max = center;
            } else {
                min = center;
            }
        }

        delta_t = min;
    }

    let ev1_plus_t = [ev1[X], ev1[Y], ev1[T] + delta_t];
    Frame::main().from_other_frame(f1, ev1_plus_t, Some(c))
}
