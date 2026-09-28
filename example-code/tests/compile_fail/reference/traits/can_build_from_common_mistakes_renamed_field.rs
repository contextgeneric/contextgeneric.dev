use cgp::core::field::impls::CanBuildFrom;
use cgp::prelude::*;

// The source spells the field `surname`; the target calls it `last_name`.
#[derive(CgpData)]
pub struct Person {
    pub first_name: String,
    pub surname: String,
}

#[derive(CgpData)]
pub struct Employee {
    pub first_name: String,
    pub last_name: String,
}

fn main() {
    let _ = Employee::builder()
        .build_from(Person {
            first_name: "Alice".to_owned(),
            surname: "Anderson".to_owned(),
        })
        .finalize_build();
}
