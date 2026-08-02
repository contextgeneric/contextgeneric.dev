//! Code from `docs/reference/traits/concat_product.md` — *`ConcatProduct`*.
//!
//! A pure type-level function, so the check is a type equality. Besides the page's worked snippet
//! this pins the two `Nil` identities it states, and the import path, since the trait is not in the
//! prelude.

/// ## Examples
pub mod examples {
    use cgp::core::field::traits::{AppendProduct, ConcatProduct};
    use cgp::prelude::*;

    pub type Base = Product![Field<Symbol!("host"), String>];

    pub type WithPort = <Base as AppendProduct<Field<Symbol!("port"), u16>>>::Output;

    pub type Extra = Product![Field<Symbol!("tls"), bool>];

    pub type Full = <WithPort as ConcatProduct<Extra>>::Output;

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
