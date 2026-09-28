//! Code from `docs/reference/types/path_cons.md` — *`PathCons`*.
//!
//! `PathCons` is built by `Path!` and emitted by `cgp_namespace!` and `#[prefix]`. Paths are markers
//! with no value to build, so the Examples program checks the chains as type equalities, including
//! a `ConcatPath` result, and registers the `cgp_namespace!` redirect the page shows. Its emitted entry
//! is checked with `cargo cgp expand`.

/// ## Examples
pub mod examples {
    use cgp::prelude::*;

    #[cgp_component(FooProvider)]
    pub trait CanFoo {
        fn foo(&self);
    }

    #[cgp_component(MyFoo)]
    pub trait CanMyFoo {
        fn my_foo(&self);
    }

    cgp_namespace! {
        new MyNamespace {
            FooProviderComponent =>
                @MyFooComponent,
        }
    }

    pub fn demo() {
        // A lowercase segment is a `Symbol`, and a capitalized one names its type.
        let _: PhantomData<PathCons<Symbol!("app"), PathCons<MyFooComponent, Nil>>> =
            PhantomData::<Path!(@app.MyFooComponent)>;

        // A one-segment path.
        let _: PhantomData<PathCons<MyFooComponent, Nil>> = PhantomData::<Path!(@MyFooComponent)>;

        // `ConcatPath` appends one path to another.
        let _: PhantomData<Path!(@a.b.c)> =
            PhantomData::<<Path!(@a.b) as ConcatPath<Path!(@c)>>::Output>;
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
