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

fn add_offsets(xy_offset: [f64; 2], t_offset: f64, ev: Event) -> Event {
    Event {
        xy: [ev.xy[0] + xy_offset[0], ev.xy[1] + xy_offset[1]],
        t: ev.t + t_offset,
    }
}

fn sub_offsets(xy_offset: [f64; 2], t_offset: f64, ev: Event) -> Event {
    add_offsets([-xy_offset[0], -xy_offset[1]], -t_offset, ev)
}

// Originally `ev` is given from a's perspective.
// returns `ev` but from b's perspective.
pub fn lorentz(a: Frame, b: Frame, ev: Event) -> Event {
    let ev = add_offsets(a.xy_offset, a.t_offset, ev);
    let ev = lorentz_impl(a.velocity, b.velocity, ev);
    let ev = sub_offsets(b.xy_offset, b.t_offset, ev);
    ev
}

fn lorentz_impl(a_v: [f64; 2], b_v: [f64; 2], ev: Event) -> Event {
    panic!()
}
