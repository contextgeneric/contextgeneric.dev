//! Compile-fail checks for the nightly snippets the pages deliberately reject.
//!
//! Each fixture under `tests/compile_fail/` is a complete program the compiler must refuse.
//! `trybuild` compiles each one and compares its output against the sibling `.stderr` file.
//! Re-bless after a toolchain or `cgp` bump with
//! `TRYBUILD=overwrite cargo test --test compile_fail_tests`.

#[test]
fn rejected_snippets_do_not_compile() {
    trybuild::TestCases::new().compile_fail("tests/compile_fail/**/*.rs");
}
