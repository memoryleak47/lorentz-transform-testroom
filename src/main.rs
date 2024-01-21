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
        // consider switching stages.
        while self.main_to_observer(self.follow_path[self.stage+1])[T] < self.t {
            if self.follow_path.get(self.stage+2).is_none() {
                return None;
            }
            self.set_stage(self.stage + 1);
        }

        let focus = self.current_pos(&self.follow_path).unwrap();

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

        self.graphics.draw(focus, pixels);

        self.t += self.tick_delta;

        Some(())
    }
}

fn main() {
    let _ = Config::load().to_ctxt().run();
}
