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
//! ```compile_fail
//! use cgp::prelude::*;
//!
//! #[cgp_component(AreaCalculator)]
//! pub trait CanCalculateArea {
//!     fn area(&self) -> f64;
//! }
//!
//! #[cgp_impl(new RectangleArea)]
//! impl AreaCalculator {
//!     fn area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
//!         width * height
//!     }
//! }
//!
//! // The mistake: no `height` field, though the provider reads one.
//! #[derive(HasField)]
//! pub struct Rectangle {
//!     pub width: f64,
//! }
//!
//! delegate_components! {
//!     Rectangle {
//!         AreaCalculatorComponent: RectangleArea,
//!     }
//! }
//!
//! check_components! {
//!     Rectangle {
//!         AreaCalculatorComponent,
//!     }
//! }
//! ```
