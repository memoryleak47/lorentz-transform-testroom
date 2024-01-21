use crate::*;

// Pathes are always given in the main frame.
pub type Path = Vec<Event>;

pub fn calc_frame(path: &[Event], stage: usize) -> Frame {
    let (start, end) = (path[stage], path[stage+1]);
    let vx = (end[X] - start[X]) / (end[T] - start[T]);
    let vy = (end[Y] - start[Y]) / (end[T] - start[T]);
    Frame { velocity: [vx, vy] }
}

