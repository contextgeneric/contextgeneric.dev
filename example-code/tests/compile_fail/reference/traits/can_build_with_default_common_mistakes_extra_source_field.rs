use cgp::extra::field::impls::CanBuildWithDefault;
use cgp::prelude::*;

// The source has a `label` field that `Point3d` does not.
#[derive(CgpData)]
pub struct LabeledPoint2d {
    pub x: u64,
    pub y: u64,
    pub label: String,
}

#[derive(CgpData)]
pub struct Point3d {
    pub x: u64,
    pub y: u64,
    pub z: u64,
}

fn main() {
    let _ = Point3d::build_with_default(LabeledPoint2d {
        x: 1,
        y: 2,
        label: "origin".to_owned(),
    });
}
