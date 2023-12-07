use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;

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

fn main() {
    let mut f = File::open("input.toml").unwrap();
    let mut data = String::new();
    f.read_to_string(&mut data).unwrap();
    let config: Config = toml::from_str(&*data).unwrap();

    for obj in config.object {
        dbg!(obj);
    }
}
