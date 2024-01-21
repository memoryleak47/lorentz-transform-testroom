use crate::*;

pub const TICK_SPEED: f64 = 0.0005;
pub const CLOCK_SPEED: f64 = 0.1;

pub struct Ctxt {
    pub follow_path: Path,
    pub pixel_objects: Vec<PixelObject>,
    pub clocks: Vec<Clock>,
    pub c: f64,
    pub last_instant: Instant,

    pub stage: usize,
    pub t: f64, // current time in the observer_frame.
    pub observer_frame: Frame,
    pub graphics: Graphics,
}

#[derive(Clone)]
pub struct PixelObject {
    pub color: u32,
    pub path: Path, // in main frame
}

#[derive(Clone)]
pub enum ClockType { Repeat, Once }

#[derive(Clone)]
pub struct Clock {
    pub ty: ClockType,
    pub path: Path,
}

impl Ctxt {
    pub fn new(follow_path: Path, pixel_objects: Vec<PixelObject>, clocks: Vec<Clock>, c: f64) -> Self {
        let mut ctxt = Ctxt {
            follow_path,
            pixel_objects,
            clocks,
            c,
            last_instant: Instant::now(),

            stage: 0,
            t: 0.0, // will be set correctly in set_stage.
            observer_frame: Frame::main(), // will be set correctly in set_stage.
            graphics: Graphics::new(),
        };

        ctxt.set_stage(0);
        ctxt
    }

    pub fn set_stage(&mut self, stage: usize) {
        self.stage = stage;
        self.observer_frame = calc_frame(&self.follow_path, stage);
        self.t = self.main_to_observer(self.follow_path[self.stage])[T];
    }

    // translates an event from the `main-frame` (eg. taken from the config file) to the observer-frame.
    pub fn main_to_observer(&self, ev: Event) -> Event {
        self.observer_frame.from_other_frame(Frame::main(), ev, Some(self.c))
    }

    pub fn find_stage(&self, path: &Path) -> Option<(usize, Event, Event)> {
        let evs: Vec<Event> = path.iter().map(|ev| self.main_to_observer(*ev)).collect();
        for i in 0..evs.len()-1 {
            if evs[i][T] <= self.t && self.t < evs[i+1][T] {
                return Some((i, evs[i], evs[i+1]));
            }
        }
        return None;
    }

    pub fn current_pos(&self, path: &Path) -> Option<[f64; 2]> {
        let (_, start, end) = self.find_stage(path)?;

        let d = (self.t - start[T]) / (end[T] - start[T]);
        let x = start[X] * (1.0 - d) + end[X] * d;
        let y = start[Y] * (1.0 - d) + end[Y] * d;
        Some([x, y])
    }

    fn local_stage_duration(&self, path: &Path, stage: usize) -> f64 {
        let f = calc_frame(path, stage);
        let (start, end) = (path[stage], path[stage+1]);
        let start = f.from_other_frame(Frame::main(), start, Some(self.c));
        let end = f.from_other_frame(Frame::main(), end, Some(self.c));

        let delta = end[T] - start[T];
        assert!(delta >= 0.0);
        delta
    }

    // returns a value from 0 to 1, representing how full the clock should be.
    pub fn clock_value(&self, clock: &Clock) -> Option<f64> {
        let (stage, start, end) = self.find_stage(&clock.path)?;
        let d = (self.t - start[T]) / (end[T] - start[T]);

        let mut sum = 0.0;
        for s in 0..stage {
            sum += self.local_stage_duration(&clock.path, s);
        }
        sum += self.local_stage_duration(&clock.path, stage) * d;
        let sum = sum * CLOCK_SPEED;

        Some(match clock.ty {
            ClockType::Once => sum.clamp(0.0, 0.5),
            ClockType::Repeat => sum % 1.0,
        })
    }
}
