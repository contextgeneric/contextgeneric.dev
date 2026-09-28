//! Code from `docs/reference/types/cons.md` — *`Cons`*.
//!
//! `Cons`/`Nil` are what `Product!` expands to. The Examples program builds a `product!` value and
//! the same value by hand, reads a struct's shape as a `Cons` chain, and folds over a list with the
//! standard pair of impls: one for `Nil` and one for `Cons<Head, Tail>`.

/// ## Examples
pub mod examples {
    use cgp::prelude::*;

    #[derive(HasFields)]
    pub struct Person {
        pub name: String,
        pub age: u8,
    }

    pub type Row = Product![u32, String, bool];

    // A fold over a product list: `Nil` is the base case, `Cons` the step.
    pub trait Len {
        const LEN: usize;
    }

    impl Len for Nil {
        const LEN: usize = 0;
    }

    impl<Head, Tail: Len> Len for Cons<Head, Tail> {
        const LEN: usize = 1 + Tail::LEN;
    }

    pub fn demo() {
        // `product!` builds the same nested value a hand-written chain does.
        let row: Row = product![1, "hi".to_owned(), true];
        let by_hand: Cons<u32, Cons<String, Cons<bool, Nil>>> =
            Cons(1, Cons("hi".to_owned(), Cons(true, Nil)));
        assert_eq!(row, by_hand);

        // A struct's shape is a `Cons` chain of `Field` entries.
        let Cons(name, Cons(age, Nil)): Cons<
            Field<Symbol!("name"), String>,
            Cons<Field<Symbol!("age"), u8>, Nil>,
        > = Person {
            name: "Alice".to_owned(),
            age: 30,
        }
        .to_fields();
        assert_eq!((name.value.as_str(), age.value), ("Alice", 30));

        assert_eq!(<Row as Len>::LEN, 3);
        assert_eq!(<<Person as HasFields>::Fields as Len>::LEN, 2);
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
