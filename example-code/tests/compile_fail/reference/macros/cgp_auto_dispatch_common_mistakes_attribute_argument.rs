//! `docs/reference/macros/cgp_auto_dispatch.md`, *Common Mistakes*: the attribute takes no
//! arguments.

use cgp::prelude::*;

#[cgp_auto_dispatch(AreaDispatcher)]
pub trait HasArea {
    fn area(&self) -> f64;
}

fn main() {}
