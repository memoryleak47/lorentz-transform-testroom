use crate::*;

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;


#[derive(Serialize, Deserialize)]
pub struct Config {
    pub c: f64,
    pub object: Vec<ConfigObject>,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone)]
pub struct ConfigObject {
    pub follow: Option<String>,
    pub clock: Option<String>,
    pub color: String,
    pub path: Path,
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
        let mut clocks = Vec::new();

        let mut follow_path: Option<Path> = None;

        const R: i32 = 20;
        for obj in &self.object {
            if let Some(s) = obj.clock.as_deref() {
                let ty = match s {
                    "once" => ClockType::Once,
                    "repeat" => ClockType::Repeat,
                    _ => panic!("invalid clock type! {}", s),
                };
                clocks.push(Clock { ty, path: obj.path.clone() });
            }

            for y in -R..=R {
                for x in -R..=R {
                    let px = mk_pixel_object(obj, x, y, self.c);

                    if x == 0 && y == 0 && obj.follow.is_some() {
                        assert!(follow_path.is_none());
                        follow_path = Some(px.path.clone());
                    }

                    pixel_objects.push(px);
                }
            }
        }

        // faster than c check!
        for pobj in &pixel_objects {
            for stage in 0..pobj.path.len()-1 {
                let v = calc_frame(&pobj.path, stage).velocity;
                let len = (v[0] * v[0] + v[1] * v[1]).sqrt();
                assert!(len < self.c, "You cannot move faster than c!");
            }
        }

        Ctxt::new(follow_path.unwrap(), pixel_objects, clocks, self.c)
    }
}

