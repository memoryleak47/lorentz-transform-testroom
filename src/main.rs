mod config;
use config::*;

mod lorentz;
use lorentz::*;

mod graphics;
use graphics::*;

struct Ctxt {
    config: Config,
    follow_obj: Object,
    stage: usize,
    t: f64,
    observer_frame: Frame,
    graphics: Graphics,
}

impl Ctxt {
    fn new() -> Ctxt {
        let config = load_config();
        assert_eq!(config.object.iter().filter(|x| x.follow.is_some()).count(), 1);
        let follow_idx = config.object.iter().position(|x| x.follow.is_some()).unwrap();

        for obj in &config.object {
            for stage in 0..obj.path.len()-1 {
                let v = Self::calc_frame(obj, stage).velocity;
                let len = (v[0] * v[0] + v[1] * v[1]).sqrt();
                assert!(len < config.c, "You cannot move faster than c!");
            }
        }


        let mut ctxt = Ctxt {
            follow_obj: config.object[follow_idx].clone(),
            config,
            stage: 0,
            t: 0.0, // will be set correctly in set_stage.
            observer_frame: Frame::main(), // will be set correctly in set_stage.
            graphics: Graphics::new(),
        };

        ctxt.set_stage(0);
        ctxt

    }

    fn run(&mut self) {
        while self.graphics.is_open() {

            // consider switching stages.
            while self.main_to_observer(self.follow_obj.path[self.stage+1])[T] < self.t {
                if self.follow_obj.path.get(self.stage+2).is_none() {
                    return;
                }
                self.set_stage(self.stage + 1);
            }

            // This is the camera center point.
            let followed_pixels = self.raw_pixels(&self.follow_obj).unwrap();
            let focus_x = followed_pixels.iter().map(|px| px.pos[0]).sum::<f64>() / followed_pixels.len() as f64;
            let focus_y = followed_pixels.iter().map(|px| px.pos[1]).sum::<f64>() / followed_pixels.len() as f64;

            let mut pixels = Vec::new();
            for obj in &self.config.object {
                if let Some(pxs) = self.raw_pixels(&obj) {
                    pixels.extend(pxs);
                }
            }

            self.graphics.draw([focus_x, focus_y], pixels);

            self.t += self.config.tick_delta;
        }
    }

    fn set_stage(&mut self, stage: usize) {
        self.stage = stage;
        self.observer_frame = Self::calc_frame(&self.follow_obj, stage);
        self.t = self.main_to_observer(self.follow_obj.path[self.stage])[T];
    }

    // translates an event from the `main-frame` (eg. taken from the config file) to the observer-frame.
    fn main_to_observer(&self, ev: Event) -> Event {
        self.observer_frame.from_other_frame(Frame::main(), ev, Some(self.config.c))
    }

    fn find_stage(&self, obj: &Object) -> Option<(usize, Event, Event)> {
        let evs: Vec<Event> = obj.path.iter().map(|ev| self.main_to_observer(*ev)).collect();
        for i in 0..evs.len()-1 {
            if evs[i][T] <= self.t && self.t < evs[i+1][T] {
                return Some((i, evs[i], evs[i+1]));
            }
        }
        return None;
    }

    fn raw_pixels(&self, obj: &Object) -> Option<Vec<Pixel>> {
        let (stage, start, end) = self.find_stage(obj)?;

        let mut pixels = Vec::new();

        let f = Self::calc_frame(obj, stage);

        // TODO is it a problem that this `d` calculation happens in the observer frame, and not in f? I think it can only happen in the observer frame though, as it requires the use of `self.t`.
        let d = (self.t - start[T]) / (end[T] - start[T]);
        let clock = self.clock_value(obj, stage, d);

        const R: i32 = 20;
        for (i, y_) in (-R..=R).enumerate() {
            for (j, x_) in (-R..=R).enumerate() {

                // calculate start & end in objs resting frame f.
                let start = f.from_other_frame(self.observer_frame, start, Some(self.config.c));
                let end = f.from_other_frame(self.observer_frame, end, Some(self.config.c));

                // add offsets there.
                let start = [start[X] + x_ as f64, start[Y] + y_ as f64, start[T]];
                let end = [end[X] + x_ as f64, end[Y] + y_ as f64, end[T]];

                // transform back to the observers frame.
                let start = self.observer_frame.from_other_frame(f, start, Some(self.config.c));
                let end = self.observer_frame.from_other_frame(f, end, Some(self.config.c));

                let d = (self.t - start[T]) / (end[T] - start[T]);
                let x = (1.0 - d) * start[X] + d * end[X];
                let y = (1.0 - d) * start[Y] + d * end[Y];

                #[allow(non_snake_case)]
                let D = (-R..=R).count();
                let ij = i*D + j;
                let max_ij = (D-1)*D + (D-1);
                let px_d = ij as f64 / max_ij as f64; // number from 0 to 1.
                assert!(px_d >= 0.0);
                assert!(px_d <= 1.0);
                let mut color = Graphics::get_color(&obj.color);
                if 1.0 - px_d <= clock {
                    color = 0xffffff;
                }
                let px = Pixel {
                    color,
                    pos: [x, y],
                };
                pixels.push(px);
            }
        }

        Some(pixels)
    }

    fn calc_frame(obj: &Object, stage: usize) -> Frame {
        let (start, end) = (obj.path[stage], obj.path[stage+1]);
        let vx = (end[X] - start[X]) / (end[T] - start[T]);
        let vy = (end[Y] - start[Y]) / (end[T] - start[T]);
        Frame { velocity: [vx, vy] }
    }

    fn local_stage_duration(&self, obj: &Object, stage: usize) -> f64 {
        let f = Self::calc_frame(obj, stage);
        let (start, end) = (obj.path[stage], obj.path[stage+1]);
        let start = f.from_other_frame(Frame::main(), start, Some(self.config.c));
        let end = f.from_other_frame(Frame::main(), end, Some(self.config.c));

        let delta = end[T] - start[T];
        assert!(delta >= 0.0);
        delta
    }

    // d is a value from 0 to 1, representing how far in the stage the object is currently.
    // returns a value from 0 to 1, representing how full the clock should be.
    fn clock_value(&self, obj: &Object, stage: usize, d: f64) -> f64 {
        if obj.clock.is_none() {
            return 0.0;
        }

        let mut sum = 0.0;
        for s in 0..stage {
            sum += self.local_stage_duration(obj, s);
        }
        sum += self.local_stage_duration(obj, stage) * d;

        let clock = (sum / self.config.max_clock).min(1.0);

        clock
    }
}

fn main() {
    Ctxt::new().run();
}
