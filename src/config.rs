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

pub fn load_config() -> Config {
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
