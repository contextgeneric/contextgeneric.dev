//! Code from `docs/reference/traits/product_ops.md` — *`AppendProduct`, `ConcatProduct` &
//! `MapFields`*.
//!
//! These are pure type-level functions, so the checks are type equalities: each `fn` coerces the
//! computed `Output`/`Mapped` type to the type the page says it produces, and the module compiles
//! only if the two are the same type. The page's import paths are checked by this file's `use`
//! lines — none of the three traits is in the prelude, nor is the `IsOptional` marker.

/// ## Examples
///
/// The append and concat snippets, as the page writes them.
pub mod examples_append_concat {
    use cgp::core::field::traits::{AppendProduct, ConcatProduct};
    use cgp::prelude::*;

    pub type Base = Product![Field<Symbol!("host"), String>];

    pub type WithPort = <Base as AppendProduct<Field<Symbol!("port"), u16>>>::Output;

    pub type Extra = Product![Field<Symbol!("tls"), bool>];

    pub type Full = <WithPort as ConcatProduct<Extra>>::Output;

    /// The page's claim about `WithPort`.
    pub fn assert_with_port(
        fields: WithPort,
    ) -> Product![Field<Symbol!("host"), String>, Field<Symbol!("port"), u16>] {
        fields
    }

    /// The page's claim about `Full`.
    pub fn assert_full(
        fields: Full,
    ) -> Product![
        Field<Symbol!("host"), String>,
        Field<Symbol!("port"), u16>,
        Field<Symbol!("tls"), bool>,
    ] {
        fields
    }

    /// The `Nil` identities the page states under *Using it*.
    pub fn assert_left_identity(fields: <Nil as ConcatProduct<Base>>::Output) -> Base {
        fields
    }

    pub fn assert_right_identity(fields: <Base as ConcatProduct<Nil>>::Output) -> Base {
        fields
    }
}

/// ## Examples
///
/// The `MapFields` snippets — over a product, and over a sum with the same marker.
pub mod examples_map_fields {
    use cgp::core::field::impls::IsOptional;
    use cgp::core::field::traits::MapFields;
    use cgp::prelude::*;

    pub type Fields = Product![String, u16, bool];

    pub type Optional = <Fields as MapFields<IsOptional>>::Mapped;

    pub fn assert_optional(
        fields: Optional,
    ) -> Product![Option<String>, Option<u16>, Option<bool>] {
        fields
    }

    pub type Variants = Sum![String, u16];

    pub type OptionalVariants = <Variants as MapFields<IsOptional>>::Mapped;

    pub fn assert_optional_variants(
        variants: OptionalVariants,
    ) -> Sum![Option<String>, Option<u16>] {
        variants
    }

    /// The page's claim that `IsPresent` is the identity, and that `IsNothing` collapses each entry.
    pub fn assert_present(fields: <Fields as MapFields<IsPresent>>::Mapped) -> Fields {
        fields
    }

    pub fn assert_nothing(fields: <Fields as MapFields<IsNothing>>::Mapped) -> Product![(), (), ()] {
        fields
    }
}
