fn sqr(x: f64) -> f64 { x * x }

// A Frame described relative to the implicit main frame with velocity = [0, 0].
// All frames represent the 'big bang' event as (t,x,y)=(0,0,0) independent of their velocities.
#[derive(Clone, Copy)]
pub struct Frame {
    pub velocity: [f64; 2],
}

pub const X: usize = 0;
pub const Y: usize = 1;
pub const T: usize = 2;
pub type Event = [f64; 3];

impl Frame {
    pub const fn main() -> Frame {
        Frame {
            velocity: [0.0, 0.0],
        }
    }

    pub fn from_other_frame(&self, other: Frame, ev: Event, c: Option<f64>) -> Event {
        // transform from other -> main.
        let ev = to_frame_with_relative_velocity([-other.velocity[0], -other.velocity[1]], ev, c);

        // transform from main -> self.
        let ev = to_frame_with_relative_velocity(self.velocity, ev, c);

        ev
    }
}

fn rotate(alpha: f64, ev: Event) -> Event {
    let alpha_cos = alpha.cos();
    let alpha_sin = alpha.sin();
    let x = ev[X] * alpha_cos - ev[Y] * alpha_sin;
    let y = ev[X] * alpha_sin + ev[Y] * alpha_cos;
    let t = ev[T];
    [x, y, t]
}

// converts `ev` from frame A to B, where A considers B to have a velocity `velocity`.
fn to_frame_with_relative_velocity(velocity: [f64; 2], ev: Event, c: Option<f64>) -> Event {
    let alpha = f64::atan2(velocity[1], velocity[0]);
    let len = f64::sqrt(sqr(velocity[0]) + sqr(velocity[1]));

    let ev = rotate(-alpha, ev);
    let ev = to_frame_with_relative_velocity_x(len, ev, c);
    let ev = rotate(alpha, ev);

    ev
}

// converts `ev` from frame A to B, where A considers B to have a velocity `[velocity_x, 0]`.
fn to_frame_with_relative_velocity_x(velocity_x: f64, ev: Event, c: Option<f64>) -> Event {
    let gamma = match c {
        Some(c) => 1.0 / f64::sqrt(1.0 - sqr(velocity_x) / sqr(c)),
        None => 1.0,
    };
    let x = gamma * (ev[X] - velocity_x * ev[T]);
    let y = ev[Y];
    let off = match c {
        Some(c) => velocity_x * ev[X] / sqr(c),
        None => 0.0,
    };
    let t = gamma * (ev[T] - off);
    [x, y, t]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(ev1: Event, ev2: Event) {
        let tolerance = 1e-5;
        assert!((ev1[X] - ev2[X]).abs() <= tolerance, "x error: {} vs. {}", ev1[X], ev2[X]);
        assert!((ev1[Y] - ev2[Y]).abs() <= tolerance, "y error: {} vs. {}", ev1[Y], ev2[Y]);
        assert!((ev1[T] - ev2[T]).abs() <= tolerance, "t error: {} vs. {}", ev1[T], ev2[T]);
    }

    const A: Frame = Frame::main();
    const B: Frame = Frame {
        velocity: [30.0, 0.0],
    };

    const C: Frame = Frame {
        velocity: [0.0, 30.0],
    };

    const EV1: Event = [2.4, 83.0, 24.0];

    #[test]
    fn transform_without_c() {
        for src in [A, B, C] {
            for dst in [A, B, C] {
                let ev = EV1;

                let x = ev[X] + ev[T] * (src.velocity[0] - dst.velocity[0]);
                let y = ev[Y] + ev[T] * (src.velocity[1] - dst.velocity[1]);
                let t = ev[T];
                let correct_ev = [x, y, t];

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
