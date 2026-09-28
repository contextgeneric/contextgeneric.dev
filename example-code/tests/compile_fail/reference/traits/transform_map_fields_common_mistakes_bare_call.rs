use cgp::core::field::traits::{TransformMap, TransformMapFields};
use cgp::prelude::*;

pub struct Keep;

impl<T> TransformMap<IsPresent, IsPresent, T> for Keep {
    fn transform_mapped(value: T) -> T {
        value
    }
}

#[derive(CgpData)]
pub struct Point {
    pub x: u32,
}

fn main() {
    // Nothing here says which transform or target marker to use.
    let _ = Point { x: 1 }.into_builder().transform_map_fields();
}
