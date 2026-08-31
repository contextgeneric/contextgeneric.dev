//! Code from `docs/reference/types/cons.md` — *`Cons`*.
//!
//! `Cons`/`Nil` are what `Product!` expands to. This pins the `Cons` chain a derive assigns and the one
//! a `Product!` alias produces, by naming the chain explicitly and by building a `product!` value by
//! hand.

/// ## Examples
///
/// A struct's shape is a `Cons` chain of `Field` entries, and a `Product!` value is a `Cons` value.
pub mod examples {
    use cgp::prelude::*;

    #[derive(HasFields)]
    pub struct Person {
        pub name: String,
        pub age: u8,
    }

    // The page shows the generated `Fields` as a `Cons` chain; the coercion checks it.
    pub fn assert_person_fields(
        fields: <Person as HasFields>::Fields,
    ) -> Cons<Field<Symbol!("name"), String>, Cons<Field<Symbol!("age"), u8>, Nil>> {
        fields
    }

    pub type Row = Product![u32, String, bool];

    // `Product!` is sugar for the right-nested `Cons` chain.
    pub fn assert_row_is_cons(row: Row) -> Cons<u32, Cons<String, Cons<bool, Nil>>> {
        row
    }

    #[test]
    fn test_a_product_value_is_a_cons_value() {
        let row: Row = product![1, "hi".to_string(), true];
        let by_hand: Cons<u32, Cons<String, Cons<bool, Nil>>> =
            Cons(1, Cons("hi".to_string(), Cons(true, Nil)));
        assert_eq!(row, by_hand);
    }
}
