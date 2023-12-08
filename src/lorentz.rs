fn sqr(x: f64) -> f64 { x * x }

// A Frame described relative to the implicit main frame.
#[derive(Clone, Copy)]
pub struct Frame {
    pub velocity: [f64; 2],
    pub t_offset: f64, // if t_offset is very high, then most events (viewed from within this frame) will have a negative t.
    pub xy_offset: [f64; 2], // if xy_offset[i] is very high, then most events (viewed from within this frame) will have a negative xy[i].
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Event {
    pub xy: [f64; 2],
    pub t: f64,
}

impl Frame {
    pub const fn main() -> Frame {
        Frame {
            velocity: [0.0, 0.0],
            t_offset: 0.0,
            xy_offset: [0.0, 0.0],
        }
    }

    pub fn from_other_frame(&self, other: Frame, ev: Event, c: Option<f64>) -> Event {
        let ev = add_offsets(other.xy_offset, other.t_offset, ev);
        let ev = lorentz_sloped(other.velocity, ev, c);
        let ev = lorentz_sloped([-self.velocity[0], -self.velocity[1]], ev, c);
        let ev = sub_offsets(self.xy_offset, self.t_offset, ev);
        ev
    }
}

fn add_offsets(xy_offset: [f64; 2], t_offset: f64, ev: Event) -> Event {
    Event {
        xy: [ev.xy[0] + xy_offset[0], ev.xy[1] + xy_offset[1]],
        t: ev.t + t_offset,
    }
}

fn sub_offsets(xy_offset: [f64; 2], t_offset: f64, ev: Event) -> Event {
    add_offsets([-xy_offset[0], -xy_offset[1]], -t_offset, ev)
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
    let alpha = f64::atan2(velocity[0], velocity[1]);
    let len = f64::sqrt(sqr(velocity[0]) + sqr(velocity[1]));

    let ev = rotate(alpha, ev);
    let ev = lorentz_straight(len, ev, c);
    let ev = rotate(-alpha, ev);

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
        Some(c) => velocity_x * ev.xy[0] / c,
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
        xy_offset: [0.0, 0.0],
        t_offset: 0.0,
    };

    const C: Frame = Frame {
        velocity: [0.0, 30.0],
        xy_offset: [0.0, 0.0],
        t_offset: 0.0,
    };

    const EV1: Event = Event {
        xy: [2.4, 83.0],
        t: -24.0,
    };

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
                let ev = dst.from_other_frame(src, EV1, Some(100.0));
                let ev = src.from_other_frame(dst, ev, Some(100.0));
                assert_close(ev, EV1);
            }
        }
    }
}
