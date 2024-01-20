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

fn dot(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[X] * b[X] + a[Y] * b[Y]
}

fn norm(a: [f64; 2]) -> f64 {
    f64::sqrt(dot(a, a))
}

fn normalized(a: [f64; 2]) -> [f64; 2] {
    let n = norm(a);
    if n == 0.0 { return [0.0, 0.0]; }
    [a[X] / n, a[Y] / n]
}

// the velocity that an object resting in f2 would have from an observer in f1.
// https://en.wikipedia.org/wiki/Relative_velocity#General_case
fn relative_velocity(f1: Frame, f2: Frame, c: f64) -> [f64; 2] {
    let va = f1.velocity;
    let vb = f2.velocity;

    let va_norm = norm(va);

    let gamma_a = 1.0 / f64::sqrt(1.0 - sqr(va_norm / c));

    let factor = 1.0 / (gamma_a * (1.0 - (dot(va, vb)/sqr(c))));
    let mut div = dot(va, vb);
    if div != 0.0 { div /= sqr(va_norm); }
    
    let rfactor = (gamma_a - 1.0) * (div - 1.0);

    [factor * (vb[X] - va[X] + va[X] * rfactor),
     factor * (vb[Y] - va[Y] + va[Y] * rfactor)]
}

// L = L0 * sqrt(1-v^2/c^2) as seen in https://en.wikipedia.org/wiki/Length_contraction
// is a number in ]0, 1]
fn contraction_factor(v_norm: f64, c: f64) -> f64 {
    f64::sqrt(1.0 - sqr(v_norm/c))
}

fn transition_event(f1: Frame, f2: Frame, ev: Event, x: i32, y: i32, c: f64) -> Event {
    // the velocity of f2 as measured from frame f1.
    let rel_v = relative_velocity(f1, f2, c);

    // If the observer stays in frame f1, by what factor would the object appear to be contracted when it is fully in f2:
    let contraction_factor = contraction_factor(norm(rel_v), c);

    // TODO: is this right? This only makes "intuitive" sense. but the units don't make sense :/
    let xy = [x as f64, y as f64];
    let directed_contraction_factor = dot(normalized(rel_v), xy) / contraction_factor;

    // the delay (as seen from f1) with which this pixel does the stage transition (delay relative to the x=0, y=0 pixel).
    let delta_t: f64 = 1.0 / contraction_factor;
    assert!(!delta_t.is_nan());

    let ev = f1.from_other_frame(Frame::main(), ev, Some(c));
    let ev = [ev[X] + x as f64, ev[Y] + y as f64, ev[T] + delta_t];
    let ev = Frame::main().from_other_frame(f1, ev, Some(c));
    ev
}
