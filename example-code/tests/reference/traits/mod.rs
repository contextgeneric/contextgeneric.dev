//! Code from the pages under `docs/reference/traits/`.
//!
//! Many pages here document traits whose impls are all generated, or which are pure type-level
//! operations with no runtime side. Those get a file only where the page shows code a compiler can
//! check: a bound that must resolve, an import path that must exist, a conversion that must run.
//! Pages that only *display* a trait definition get no module, since re-declaring the trait would
//! check nothing about CGP.

pub mod append_product;
pub mod build_field;
pub mod can_build_from;
pub mod can_build_with_default;
pub mod can_downcast;
pub mod can_downcast_fields;
pub mod can_finalize_with_default;
pub mod can_upcast;
pub mod can_use_component;
pub mod concat_path;
pub mod concat_product;
pub mod contains_value;
pub mod default_impls1;
pub mod default_impls2;
pub mod default_namespace;
pub mod delegate_component;
pub mod extract_field;
pub mod field_getter;
pub mod field_mapper;
pub mod finalize_build;
pub mod finalize_extract;
pub mod finalize_extract_result;
pub mod finalize_optional;
pub mod from_fields;
pub mod from_variant;
pub mod has_builder;
pub mod has_extractor;
pub mod has_extractor_mut;
pub mod has_extractor_ref;
pub mod has_field;
pub mod has_field_mut;
pub mod has_fields;
pub mod has_fields_ref;
pub mod has_optional_builder;
pub mod into_builder;
pub mod is_provider_for;
pub mod lift_value;
pub mod map_field;
pub mod map_fields;
pub mod map_type;
pub mod map_type_ref;
pub mod monadic_bind;
pub mod monadic_trans;
pub mod mut_field_getter;
pub mod partial_data;
pub mod set_optional;
pub mod static_format;
pub mod static_string;
pub mod take_field;
pub mod to_fields;
pub mod to_fields_ref;
pub mod to_optional;
pub mod transform_map;
pub mod transform_map_default;
pub mod transform_map_fields;
pub mod transform_optional;
pub mod update_field;
