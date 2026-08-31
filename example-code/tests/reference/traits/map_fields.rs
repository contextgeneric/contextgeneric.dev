//! Code from `docs/reference/traits/map_fields.md` — *`MapFields`*.
//!
//! A pure type-level function, so the check is a type equality. What this pins beyond the page's two
//! snippets is that the same marker applies over *both* lists — a product and a sum — and the two
//! import paths, since neither the trait nor the `IsOptional` marker is in the prelude.

/// ## Examples
pub mod examples {
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
