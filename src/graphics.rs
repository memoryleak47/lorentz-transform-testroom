use minifb::{Key, Window, WindowOptions};
use crate::*;

const WIDTH: usize = 1800;
const HEIGHT: usize = 1200;

pub struct Graphics {
    buffer: Vec<u32>,
    window: Window,
}

pub struct Pixel {
    pub pos: [f64; 2],
    pub color: u32,
}

impl Graphics {
    pub fn new() -> Graphics {
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

        Graphics { buffer, window }
    }

    pub fn draw(&mut self, focus: [f64; 2], pixels: Vec<Pixel>) {
        self.buffer.iter_mut().for_each(|x| *x = 0);

        // render objects.
        for px in pixels {
            let x: f64 = px.pos[X] - focus[X] + WIDTH as f64/2.0;
            let y: f64 = px.pos[Y] - focus[Y] + HEIGHT as f64/2.0;
            if x < 0.0 || x > WIDTH as f64 { continue; }
            if y < 0.0 || y > HEIGHT as f64 { continue; }
            self.buffer[x as usize + y as usize * WIDTH] = px.color;
        }

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        self.window
            .update_with_buffer(&self.buffer, WIDTH, HEIGHT)
            .unwrap();

    }

    pub fn is_open(&self) -> bool {
        self.window.is_open() && !self.window.is_key_down(Key::Escape)
    }
}

pub fn get_color(s: &str) -> u32 {
    match s {
        "red" => 0xff0000,
        "blue" => 0x0000ff,
        "green" => 0x00ff00,
        "yellow" => 0xffff00,
        "violet" => 0xff00ff,
        _ => panic!(),
    }
}

