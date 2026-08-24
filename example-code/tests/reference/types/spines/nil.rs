//! Code from `docs/reference/types/spines/nil.md` — *`Nil`*.
//!
//! `Nil` terminates the product, string, and path spines and is the empty product. This pins that the
//! empty `Product!` is `Nil`, at the type level and the value level.

/// ## Examples
///
/// A `Cons` chain ends in `Nil`, and the empty product is `Nil` alone.
pub mod examples {
    use cgp::prelude::*;

    pub type Row = Cons<u32, Cons<bool, Nil>>;

    pub type Empty = Product![];

    pub fn assert_empty_is_nil(empty: Empty) -> Nil {
        empty
    }

    #[test]
    fn test_the_empty_product_value_is_nil() {
        let empty: Empty = product![];
        assert_eq!(empty, Nil);
    }
}
