//! `docs/reference/macros/cgp_type.md`, *Common mistakes*: the associated type may not be generic.

use cgp::prelude::*;

#[cgp_type]
pub trait HasContainerType {
    type Container<T>;
}

fn main() {}
