//! Code from `docs/reference/traits/append_product.md` — *`AppendProduct`*.
//!
//! A pure type-level function, so the check is a type equality: each `fn` coerces the computed
//! `Output` to the type the page says it produces, and the module compiles only if the two are the
//! same type. The `use` line checks the import path, since the trait is not in the prelude.

/// ## Examples
pub mod examples {
    use cgp::core::field::traits::AppendProduct;
    use cgp::prelude::*;

    pub type Base = Product![Field<Symbol!("host"), String>];

    pub type WithPort = <Base as AppendProduct<Field<Symbol!("port"), u16>>>::Output;

    /// The page's claim about `WithPort`.
    pub fn assert_with_port(
        fields: WithPort,
    ) -> Product![Field<Symbol!("host"), String>, Field<Symbol!("port"), u16>] {
        fields
    }

    /// The base case the page states under *Using it*: appending onto `Nil` yields a one-element list.
    pub fn assert_nil_base_case(
        fields: <Nil as AppendProduct<Field<Symbol!("host"), String>>>::Output,
    ) -> Base {
        fields
    }
}
