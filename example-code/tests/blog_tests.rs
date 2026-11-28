//! Compiled counterparts of blog-post examples.
#[path = "blog/v0_8_0_release.rs"]
mod v0_8_0_release;

#[path = "blog/incoherent_rust_today.rs"]
mod incoherent_rust_today;

#[test]
fn missing_height_is_rejected() {
    trybuild::TestCases::new().compile_fail("tests/compile_fail/blog/v0_8_0_missing_height.rs");
}
