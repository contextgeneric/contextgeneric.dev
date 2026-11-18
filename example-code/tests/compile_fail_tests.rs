//! Compile-fail checks for the snippets the docs pages deliberately reject.
//!
//! Each fixture under `tests/compile_fail/` is a complete program the compiler must refuse, carrying
//! the rejected snippet with its elided bodies filled in so the intended error is the only thing
//! wrong with it. `trybuild` compiles each one and compares its output against the sibling `.stderr`
//! file; a fixture that has started compiling is the regression this guards against.
//!
//! Re-bless the `.stderr` files after a toolchain or `cgp` bump with
//! `TRYBUILD=overwrite cargo test --test compile_fail_tests`.

#[test]
fn rejected_snippets_do_not_compile() {
    trybuild::TestCases::new().compile_fail("tests/compile_fail/**/*.rs");
}
