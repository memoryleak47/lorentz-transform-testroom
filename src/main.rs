use std::time::Instant;

mod config;
use config::*;

mod ctxt;
use ctxt::*;

mod path;
use path::*;

mod lorentz;
use lorentz::*;

mod graphics;
use graphics::*;

mod pixelate;
use pixelate::*;

impl Ctxt {
    fn run(&mut self) -> Option<()> {
        while self.graphics.is_open() {
            self.tick()?;
        }
        None
    }

    fn tick(&mut self) -> Option<()> {
        let elapsed = self.last_instant.elapsed();
        self.last_instant = Instant::now();

        // consider switching stages.
        while self.main_to_observer(self.follow_path[self.stage+1])[T] < self.t {
            if self.follow_path.get(self.stage+2).is_none() {
                return None;
            }
            self.set_stage(self.stage + 1);
        }

        let focus = self.current_pos(&self.follow_path).unwrap();

        let mut pixels = self.draw_pixel_objects();
        pixels.extend(self.draw_clocks());

        self.graphics.draw(focus, pixels);

        let seconds = elapsed.as_micros() as f64 / 1000.0 / 1000.0;
        self.t += seconds * TICK_SPEED;

        Some(())
    }

    fn draw_pixel_objects(&self) -> Vec<Pixel> {
        let mut pixels = Vec::new();
        for pobj in &self.pixel_objects {
            if let Some(px) = self.current_pos(&pobj.path) {
                let stage_parity = self.find_stage(&pobj.path).unwrap().0 % 2;
                let px = Pixel {
                    pos: px,
                    color: pobj.color + (stage_parity * 160) as u32,
                };
                pixels.push(px);
            }
        }
        pixels
    }

    fn draw_clocks(&self) -> Vec<Pixel> {
        fn clock_color(x: f64, y: f64, val: f64) -> bool {
            // adds frame around the clock.
            const F: f64 = 0.85 * (CLOCK_RADIUS as f64);
            if x*x + y*y > F*F { return true; }

            // in [0, 2*pi]
            let angle = (-x).atan2(y) + std::f64::consts::PI;

            if val >= 0.5 {
                // in [0, 2*pi]
                let pi_val = (val - 0.5) * 4.0 * std::f64::consts::PI;

                angle < pi_val
            } else {
                // in [0, 2*pi]
                let pi_val = val * 4.0 * std::f64::consts::PI;

                angle > pi_val
            }
        }

        let mut pixels = Vec::new();
        for cl in &self.clocks {
            if let Some(pos) = self.current_pos(&cl.path) {
                let clock_val = self.clock_value(&cl).unwrap();
                for x in -CLOCK_RADIUS..=CLOCK_RADIUS {
                    for y in -CLOCK_RADIUS..=CLOCK_RADIUS {
                        let (x, y) = (x as f64, y as f64);

                        // this check makes the clock round.
                        if x*x + y*y > (CLOCK_RADIUS * CLOCK_RADIUS) as f64 { continue; }

                        let color = match clock_color(x, y, clock_val) {
                            true => 0x333333,
                            false => 0xdddddd,
                        };

                        let px = Pixel {
                            pos: [pos[X] + x, pos[Y] + y],
                            color,
                        };
                        pixels.push(px);
                    }
                }
            }
        }
        pixels
    }
}

fn main() {
    let _ = Config::load().to_ctxt().run();
}
