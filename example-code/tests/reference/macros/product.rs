//! Code from `docs/reference/macros/product.md` — *`Product!` & `product!`*.
//!
//! The snippets the page rejects live under `tests/compile_fail/reference/macros/`. Type
//! equalities are checked by assigning one `PhantomData` to a binding typed with the other.

/// ## Usage
///
/// The type, its value, the empty list, and a trailing comma.
pub mod usage {
    use cgp::prelude::*;

    #[test]
    fn the_value_has_the_type_the_type_macro_builds() {
        let row: Product![u32, String, bool] = product![1u32, "hi".to_string(), true];
        let Cons(first, Cons(_, Cons(third, Nil))) = row;
        assert_eq!((first, third), (1, true));

        let _: PhantomData<Product![]> = PhantomData::<Nil>;
        let _: PhantomData<Product![u32, bool,]> = PhantomData::<Product![u32, bool]>;
        let Nil = product![];
    }
}

/// ## Examples
///
/// The derived field list, the pipeline the page wires, and the standalone row. The page names
/// `MyContext`, `Multiply`, and `Add` without declaring them; `Multiply<Tag>` and `Add<Tag>` are
/// computers reading a `u64` field named by `Tag`.
pub mod examples {
    use cgp::extra::handler::{CanCompute, Computer, ComputerComponent, PipeHandlers};
    use cgp::prelude::*;

    #[derive(HasFields)]
    pub struct Person {
        pub name: String,
        pub age: u8,
    }

    #[test]
    fn the_derive_generates_the_field_list() {
        let _: PhantomData<<Person as HasFields>::Fields> = PhantomData::<
            Product![Field<Symbol!("name"), String>, Field<Symbol!("age"), u8>],
        >;
    }

    #[cgp_impl(new Multiply<Tag>)]
    #[uses(HasField<Tag, Value = u64>)]
    impl<Code, Tag> Computer<Code, u64> {
        type Output = u64;

        fn compute(&self, _code: PhantomData<Code>, input: u64) -> u64 {
            input * self.get_field(PhantomData::<Tag>)
        }
    }

    #[cgp_impl(new Add<Tag>)]
    #[uses(HasField<Tag, Value = u64>)]
    impl<Code, Tag> Computer<Code, u64> {
        type Output = u64;

        fn compute(&self, _code: PhantomData<Code>, input: u64) -> u64 {
            input + self.get_field(PhantomData::<Tag>)
        }
    }

    #[derive(HasField)]
    pub struct MyContext {
        pub foo: u64,
        pub bar: u64,
        pub baz: u64,
    }

    delegate_components! {
        MyContext {
            ComputerComponent:
                PipeHandlers<Product![
                    Multiply<Symbol!("foo")>,
                    Add<Symbol!("bar")>,
                    Multiply<Symbol!("baz")>,
                ]>,
        }
    }

    check_components! {
        MyContext {
            ComputerComponent: ((), u64),
        }
    }

    #[test]
    fn the_steps_run_left_to_right() {
        let context = MyContext {
            foo: 2,
            bar: 3,
            baz: 4,
        };
        assert_eq!(context.compute(PhantomData::<()>, 5), ((5 * 2) + 3) * 4);
    }

    #[test]
    fn a_standalone_row() {
        type Row = Product![u32, String, bool];
        let row: Row = product![1, "hi".to_string(), true];
        let Cons(first, _) = row;
        assert_eq!(first, 1);
    }
}

/// ## Under the hood
pub mod under_the_hood {
    use cgp::prelude::*;

    pub struct A;
    pub struct B;
    pub struct C;

    #[test]
    fn both_macros_fold_onto_nil() {
        let _: PhantomData<Product![A, B, C]> = PhantomData::<Cons<A, Cons<B, Cons<C, Nil>>>>;
        let Cons(A, Cons(B, Cons(C, Nil))) = product![A, B, C];
    }
}

/// ## Common Mistakes
///
/// A one-element list is a distinct type from its element. The misplaced-macro snippets are
/// trybuild fixtures.
pub mod common_mistakes {
    use cgp::prelude::*;

    #[test]
    fn a_one_element_list_wraps_its_element() {
        let one: Product![u32] = product![7];
        let Cons(inner, Nil) = one;
        assert_eq!(inner, 7);
    }
}
