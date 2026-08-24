//! Code from the pages under `docs/reference/types/`.
//!
//! Most of these types are type-level markers a reader recognizes rather than writes, so a page gets a
//! module only where it shows code a compiler can check: a shape a derive assigns, a value built from a
//! marker, a call site that passes a tag. The two overview pages, which show no code, get none.

pub mod field;
pub mod index_type;
pub mod life;
pub mod mref;
pub mod phantom_data;
pub mod spines;
