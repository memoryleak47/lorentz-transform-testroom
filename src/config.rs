use crate::*;

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;


#[derive(Serialize, Deserialize)]
pub struct Config {
    pub c: f64,
    pub tick_delta: f64,
    pub object: Vec<ConfigObject>,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone)]
pub struct ConfigObject {
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

