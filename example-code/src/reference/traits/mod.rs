//! Code from the pages under `docs/reference/traits/`.
//!
//! Several pages here document traits whose impls are all generated, or which are pure type-level
//! operations with no runtime side. Those get a file only where the page shows code a compiler can
//! check: a bound that must resolve, an import path that must exist, a conversion that must run.
//! Pages that only *display* a trait definition get no module, since re-declaring the trait would
//! check nothing about CGP.

pub mod cast;
pub mod has_builder;
pub mod product_ops;
pub mod static_format;
