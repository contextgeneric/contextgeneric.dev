//! Code from `docs/reference/macros/sum.md` — *`Sum!`*.
//!
//! The snippet the page rejects lives under `tests/compile_fail/reference/macros/`. Type
//! equalities are checked by assigning one `PhantomData` to a binding typed with the other.

/// ## Usage
pub mod usage {
    use cgp::prelude::*;

    #[test]
    fn the_empty_sum_is_void() {
        let _: PhantomData<Sum![]> = PhantomData::<Void>;
        let _: PhantomData<Sum![u32, bool,]> = PhantomData::<Sum![u32, bool]>;
    }
}

/// ## Examples
///
/// The enum whose derived variant list the page shows, extended with the two payload shapes the
/// page describes in prose: several unnamed fields, keyed by `Index`, and a unit variant.
pub mod examples {
    use cgp::prelude::*;

    #[derive(HasFields)]
    pub enum Shape {
        Circle(f64),
        Rectangle { width: f64, height: f64 },
    }

    #[test]
    fn the_derive_generates_the_variant_list() {
        let _: PhantomData<<Shape as HasFields>::Fields> = PhantomData::<
            Sum![
                Field<Symbol!("Circle"), f64>,
                Field<
                    Symbol!("Rectangle"),
                    Product![Field<Symbol!("width"), f64>, Field<Symbol!("height"), f64>],
                >,
            ],
        >;
    }

    #[derive(HasFields)]
    pub enum Payloads {
        Pair(f64, f64),
        Empty,
    }

    #[test]
    fn unnamed_fields_nest_by_index_and_a_unit_variant_holds_nil() {
        let _: PhantomData<<Payloads as HasFields>::Fields> = PhantomData::<
            Sum![
                Field<Symbol!("Pair"), Product![Field<Index<0>, f64>, Field<Index<1>, f64>]>,
                Field<Symbol!("Empty"), Nil>,
            ],
        >;
    }

    pub type Token = Sum![u32, String, bool];
}

/// ## Under the hood
///
/// The `Either`/`Void` chain, and the position of each alternative in it.
pub mod under_the_hood {
    use cgp::prelude::*;

    pub struct A;
    pub struct B;
    pub struct C;

    #[test]
    fn the_sum_folds_onto_void() {
        let _: PhantomData<Sum![A, B, C]> =
            PhantomData::<Either<A, Either<B, Either<C, Void>>>>;

        let b: Sum![A, B, C] = Either::Right(Either::Left(B));
        assert!(matches!(b, Either::Right(Either::Left(B))));
    }

    /// A handled-every-variant remainder is `Void`, discharged without a fallback arm.
    pub fn exhausted(remainder: Void) -> u8 {
        match remainder {}
    }
}
