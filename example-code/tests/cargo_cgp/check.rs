//! Code from `docs/cargo-cgp/check.md` — *Check*, and the excerpt of it on the section index.
//!
//! The page's whole program is the mistake: `RectangleArea` reads a `height` field through an
//! `#[implicit]` argument and `Rectangle` does not have one. It therefore cannot be a module, and is
//! carried as a `compile_fail` doctest.
//!
//! What the doctest guards is narrow but real: that the snippet the page calls broken is still broken.
//! It does **not** verify which diagnostic comes out — rustdoc does not enforce that — and it says
//! nothing at all about the `cargo cgp check` output the page quotes beside it. That output was
//! produced by running the tool, and re-running it is the only way to re-verify it.
//!
//! Rejected snippet — trybuild fixture `tests/compile_fail/cargo_cgp/check_check.rs`.
