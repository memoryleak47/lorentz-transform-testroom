use serde::*;

fn sqr(x: f64) -> f64 { x * x }

// A Frame described relative to the implicit main frame with velocity = [0, 0].
// All frames represent the 'big bang' event as (t,x,y)=(0,0,0) independent of their velocities.
#[derive(Clone, Copy)]
pub struct Frame {
    pub velocity: [f64; 2],
}

#[derive(Clone, Copy, PartialEq, Debug)]
#[derive(Serialize, Deserialize)]
pub struct Event {
    pub xy: [f64; 2],
    pub t: f64,
}

impl Frame {
    pub const fn main() -> Frame {
        Frame {
            velocity: [0.0, 0.0],
        }
    }

    pub fn from_other_frame(&self, other: Frame, ev: Event, c: Option<f64>) -> Event {
        // transform from other -> main.
        let ev = lorentz_sloped(other.velocity, ev, c);

        // transform from main -> self.
        let ev = lorentz_sloped([-self.velocity[0], -self.velocity[1]], ev, c);

        ev
    }
}

fn rotate(alpha: f64, ev: Event) -> Event {
    let alpha_cos = alpha.cos();
    let alpha_sin = alpha.sin();
    let x = ev.xy[0] * alpha_cos - ev.xy[1] * alpha_sin;
    let y = ev.xy[0] * alpha_sin + ev.xy[1] * alpha_cos;
    let t = ev.t;
    Event {
        xy: [x, y],
        t,
    }
}

fn lorentz_sloped(velocity: [f64; 2], ev: Event, c: Option<f64>) -> Event {
    let alpha = f64::atan2(velocity[1], velocity[0]);
    let len = f64::sqrt(sqr(velocity[0]) + sqr(velocity[1]));

    let ev = rotate(-alpha, ev);
    let ev = lorentz_straight(-len, ev, c);
    let ev = rotate(alpha, ev);

    ev
}

fn lorentz_straight(velocity_x: f64, ev: Event, c: Option<f64>) -> Event {
    let gamma = match c {
        Some(c) => 1.0 / f64::sqrt(1.0 - sqr(velocity_x) / sqr(c)),
        None => 1.0,
    };
    let x = gamma * (ev.xy[0] - velocity_x * ev.t);
    let y = ev.xy[1];
    let off = match c {
        Some(c) => velocity_x * ev.xy[0] / sqr(c),
        None => 0.0,
    };
    let t = gamma * (ev.t - off);
    Event {
        xy: [x, y],
        t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(ev1: Event, ev2: Event) {
        const T: f64 = 1e-5;
        assert!((ev1.xy[0] - ev2.xy[0]).abs() <= T, "x error: {} vs. {}", ev1.xy[0], ev2.xy[0]);
        assert!((ev1.xy[1] - ev2.xy[1]).abs() <= T, "y error: {} vs. {}", ev1.xy[1], ev2.xy[1]);
        assert!((ev1.t - ev2.t).abs() <= T, "t error: {} vs. {}", ev1.t, ev2.t);
    }

    const A: Frame = Frame::main();
    const B: Frame = Frame {
        velocity: [30.0, 0.0],
    };

    const C: Frame = Frame {
        velocity: [0.0, 30.0],
    };

    const EV1: Event = Event {
        xy: [2.4, 83.0],
        t: 24.0,
    };

    #[test]
    fn transform_without_c() {
        for src in [A, B, C] {
            for dst in [A, B, C] {
                let ev = EV1;

                let x = ev.xy[0] + ev.t * (src.velocity[0] - dst.velocity[0]);
                let y = ev.xy[1] + ev.t * (src.velocity[1] - dst.velocity[1]);
                let t = ev.t;

                let correct_ev = Event {
                    xy: [x, y],
                    t,
                };

                let ev = dst.from_other_frame(src, ev, None);
                assert_close(ev, correct_ev);
            }
        }
    }

    #[test]
    fn refl_test() {
        for frame in [A, B, C] {
            let ev = EV1;
            let ev = frame.from_other_frame(frame, ev, Some(100.0));
            assert_close(ev, EV1);
        }
    }

    #[test]
    fn reversibility() {
        for src in [A, B, C] {
            for dst in [A, B, C] {
                let ev = EV1;
                let ev = dst.from_other_frame(src, ev, Some(100.0));
                let ev = src.from_other_frame(dst, ev, Some(100.0));
                assert_close(ev, EV1);
            }
        }
    }
}
