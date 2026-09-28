use cgp::prelude::*;
use serde::Serialize;

// error: cannot find attribute `serde` in this scope
//
// The companion `__PartialPerson` copies each field's attributes but not the struct's derives, so
// the `serde` helper lands on a struct that does not derive `Serialize`.
#[derive(Serialize, BuildField)]
pub struct Person {
    #[serde(rename = "name")]
    pub first_name: String,
}

fn main() {}
