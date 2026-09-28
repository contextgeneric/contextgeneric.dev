//! Code from `docs/reference/types/nil.md` — *`Nil`*.
//!
//! `Nil` terminates the product, string, and path lists and is the empty product. The Examples
//! program checks each of those as a type equality, and builds the empty product and a unit struct's
//! shape as `Nil` values.

/// ## Examples
pub mod examples {
    use cgp::prelude::*;

    #[derive(HasFields)]
    pub struct Marker;

    pub fn demo() {
        // Each list ends in `Nil`; the annotations are the check.
        let _row: PhantomData<Cons<u32, Cons<bool, Nil>>> = PhantomData::<Product![u32, bool]>;
        let _name: PhantomData<Symbol<2, Chars<'h', Chars<'i', Nil>>>> =
            PhantomData::<Symbol!("hi")>;
        let _path: PhantomData<PathCons<Symbol!("a"), PathCons<Symbol!("b"), Nil>>> =
            PhantomData::<Path!(@a.b)>;

        // The empty product, the empty string, and a unit struct's shape are all `Nil`.
        let empty: Product![] = product![];
        assert_eq!(empty, Nil);
        let _empty_name: PhantomData<Symbol<0, Nil>> = PhantomData::<Symbol!("")>;
        let fields: Nil = Marker.to_fields();
        assert_eq!(fields, Nil);
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
