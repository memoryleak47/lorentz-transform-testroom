use crate::*;

// from the rest frame of an object, it is spawned "at once", and it dies "at once".
// within a stage change, the acceleration time delta can be calculated using the required length contraction in either frame.
pub fn mk_pixel_object(object: &ConfigObject, x: i32, y: i32, c: f64) -> PixelObject {
    let color = get_color(&object.color);

    let mut path = Vec::new();

    // start event
    let start_frame = calc_frame(&object.path, 0);
    let start_ev = *object.path.first().unwrap();
    path.push(simultaneous_event(start_frame, start_ev, x, y, c));

    // transition events
    for i in 0..object.path.len() - 2 {
        let f1 = calc_frame(&object.path, i);
        let f2 = calc_frame(&object.path, i+1);
        let ev = object.path[i+1];
        path.push(transition_event(f1, f2, ev, x, y, c));
    }

    // end event
    let end_frame = calc_frame(&object.path, object.path.len() - 2);
    let end_ev = *object.path.last().unwrap();
    path.push(simultaneous_event(end_frame, end_ev, x, y, c));

    PixelObject {
        path,
        color,
    }
}

fn simultaneous_event(f: Frame, ev: Event, x: i32, y: i32, c: f64) -> Event {
    let ev = f.from_other_frame(Frame::main(), ev, Some(c));
    let ev = [ev[X] + x as f64, ev[Y] + y as f64, ev[T]];
    let ev = Frame::main().from_other_frame(f, ev, Some(c));
    ev
}

fn transition_event(f1: Frame, f2: Frame, ev: Event, x: i32, y: i32, c: f64) -> Event {
    let sqr = |x| x*x;
    let ev1 = f1.from_other_frame(Frame::main(), ev, Some(c));
    let ev1 = [ev1[X] + x as f64, ev1[Y] + y as f64, ev1[T]];

    let ev2 = f2.from_other_frame(Frame::main(), ev, Some(c));
    let ev2 = [ev2[X] + x as f64, ev2[Y] + y as f64, ev2[T]];

    let mut delta_t = 0.0;

    // TODO: doing this with a closed-form equation shouldn't be so hard. Try it.

    // If f1 = f2, we don't need delta_t. It's the same frame.
    // ... also, our binary search breaks down because the system is underspecified.
    if f1.velocity != f2.velocity {
        // Note that in both frames f1 and f2, the X and Y components will not change, as the object itself is at rest there.
        // Hence, when switching from f1 to f2, we have to make sure that initially,
        // (right when the switch happens), the coordinates are already correct.
        // We can use this to determine the correct point in time, when the switch should happen.
        //
        // Thus, we want to find a `delta_t`, s.t. ev1 offseted by delta_t converted to frame f2, is equal to ev2 in X and Y components.
        // we do that using binary search, we want to find a t with minimal t_dist.
        let t_dist = |t| {
            let ev1_plus_t = [ev1[X], ev1[Y], ev1[T] + t];
            let ev2_ = f2.from_other_frame(f1, ev1_plus_t, Some(c));

            sqr(ev2_[X] - ev2[X]) + sqr(ev2_[Y] - ev2[Y])
        };

        let mut min = -10000.0;
        let mut max = 10000.0;

        for _ in 0..100 {
            let center = (min + max)/2.0;
            if t_dist(min) < t_dist(max) {
                max = center;
            } else {
                min = center;
            }
        }

        delta_t = min;
    }

    let ev1_plus_t = [ev1[X], ev1[Y], ev1[T] + delta_t];
    Frame::main().from_other_frame(f1, ev1_plus_t, Some(c))
}
