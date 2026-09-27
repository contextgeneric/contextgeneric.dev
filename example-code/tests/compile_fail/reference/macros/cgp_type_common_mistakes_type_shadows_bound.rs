//! `docs/reference/macros/cgp_type.md`, *Common Mistakes*: an associated type named after the trait
//! that bounds it shadows that trait.

use cgp::prelude::*;

pub trait Database {}

#[cgp_type]
pub trait HasDatabaseType {
    type Database: Database;
}

fn main() {}
