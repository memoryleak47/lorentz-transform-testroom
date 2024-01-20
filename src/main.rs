mod config;
use config::*;

mod lorentz;
use lorentz::*;

mod graphics;
use graphics::*;

struct Ctxt {
    follow_obj: PixelObject,
    pixel_objects: Vec<PixelObject>,
    c: f64,
    tick_delta: f64,

    stage: usize,
    t: f64, // current time in the observer_frame.
    observer_frame: Frame,
    graphics: Graphics,
}

#[derive(Clone)]
struct PixelObject {
    color: u32,
    path: Vec<Event>, // in main frame
}

impl Ctxt {
    fn run(&mut self) -> Option<()> {
        while self.graphics.is_open() {
            self.tick()?;
        }
        None
    }

    fn tick(&mut self) -> Option<()> {
        // consider switching stages.
        while self.main_to_observer(self.follow_obj.path[self.stage+1])[T] < self.t {
            if self.follow_obj.path.get(self.stage+2).is_none() {
                return None;
            }
            self.set_stage(self.stage + 1);
        }

        let focus = self.current_pos(&self.follow_obj).unwrap();

        let mut pixels = Vec::new();
        for pobj in &self.pixel_objects {
            if let Some(px) = self.current_pos(&pobj) {
                let stage_parity = self.find_stage(&pobj).unwrap().0 % 2;
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

    fn set_stage(&mut self, stage: usize) {
        self.stage = stage;
        self.observer_frame = Self::calc_frame(&self.follow_obj.path, stage);
        self.t = self.main_to_observer(self.follow_obj.path[self.stage])[T];
    }

    // translates an event from the `main-frame` (eg. taken from the config file) to the observer-frame.
    fn main_to_observer(&self, ev: Event) -> Event {
        self.observer_frame.from_other_frame(Frame::main(), ev, Some(self.c))
    }

    fn find_stage(&self, pixel_obj: &PixelObject) -> Option<(usize, Event, Event)> {
        let evs: Vec<Event> = pixel_obj.path.iter().map(|ev| self.main_to_observer(*ev)).collect();
        for i in 0..evs.len()-1 {
            if evs[i][T] <= self.t && self.t < evs[i+1][T] {
                return Some((i, evs[i], evs[i+1]));
            }
        }
        return None;
    }

    fn current_pos(&self, pixel_obj: &PixelObject) -> Option<[f64; 2]> {
        let (_, start, end) = self.find_stage(pixel_obj)?;

        let d = (self.t - start[T]) / (end[T] - start[T]);
        let x = start[X] * (1.0 - d) + end[X] * d;
        let y = start[Y] * (1.0 - d) + end[Y] * d;
        Some([x, y])
    }

    fn calc_frame(path: &[Event], stage: usize) -> Frame {
        let (start, end) = (path[stage], path[stage+1]);
        let vx = (end[X] - start[X]) / (end[T] - start[T]);
        let vy = (end[Y] - start[Y]) / (end[T] - start[T]);
        Frame { velocity: [vx, vy] }
    }
}

fn main() {
    let _ = Config::load().to_ctxt().run();
}
