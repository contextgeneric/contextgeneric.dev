use cgp::core::field::impls::CanBuildFrom;
use cgp::prelude::*;

#[derive(CgpData)]
pub struct FooBar {
    pub foo: u64,
    pub bar: String,
}

#[derive(CgpData)]
pub struct FooBaz {
    pub foo: u64,
    pub baz: bool,
}

#[derive(CgpData)]
pub struct FooBarBaz {
    pub foo: u64,
    pub bar: String,
    pub baz: bool,
}

fn main() {
    // Both sources carry `foo`, so the second merge would set it twice.
    let _ = FooBarBaz::builder()
        .build_from(FooBar { foo: 1, bar: "bar".to_owned() })
        .build_from(FooBaz { foo: 2, baz: true })
        .finalize_build();
}
