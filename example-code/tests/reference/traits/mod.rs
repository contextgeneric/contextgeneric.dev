//! Code from the pages under `docs/reference/traits/`.
//!
//! Many pages here document traits whose impls are all generated, or which are pure type-level
//! operations with no runtime side. Those get a file only where the page shows code a compiler can
//! check: a bound that must resolve, an import path that must exist, a conversion that must run.
//! Pages that only *display* a trait definition get no module, since re-declaring the trait would
//! check nothing about CGP.

pub mod append_product;
pub mod can_build_from;
pub mod can_downcast;
pub mod can_downcast_fields;
pub mod can_upcast;
pub mod concat_path;
pub mod concat_product;
pub mod has_builder;
pub mod map_fields;
pub mod static_format;
pub mod static_string;
