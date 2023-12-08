fn sqr(x: f64) -> f64 { x * x }

// A Frame described relative to the implicit main frame.
#[derive(Clone, Copy)]
pub struct Frame {
    pub velocity: [f64; 2],
    pub t_offset: f64, // if t_offset is very high, then most events (viewed from within this frame) will have a negative t.
    pub xy_offset: [f64; 2], // if xy_offset[i] is very high, then most events (viewed from within this frame) will have a negative xy[i].
}

#[derive(Clone, Copy)]
pub struct Event {
    pub xy: [f64; 2],
    pub t: f64,
}

impl Frame {
    pub fn main() -> Frame {
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