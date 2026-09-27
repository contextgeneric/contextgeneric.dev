//! Code from `docs/reference/macros/path.md` — *`Path!`*.
//!
//! The snippets the page rejects live under `tests/compile_fail/reference/macros/`. Type
//! equalities are checked by assigning one `PhantomData` to a binding typed with the other.

/// ## Usage and How a segment is encoded
///
/// The three example paths, and one segment of each kind in the page's table.
pub mod usage {
    use cgp::core::error::ErrorRaiserComponent;
    use cgp::prelude::*;

    pub type One = Path!(@app);
    pub type Two = Path!(@app.error);
    pub type Three = Path!(@app.error.ErrorRaiserComponent);

    pub struct ShowImplComponent;

    pub mod some_mod {
        pub struct Marker;
    }

    #[test]
    fn a_segment_is_encoded_by_its_spelling() {
        // A lowercase identifier becomes a symbol.
        let _: PhantomData<Path!(@app)> = PhantomData::<PathCons<Symbol!("app"), Nil>>;
        let _: PhantomData<Path!(@my_app)> = PhantomData::<PathCons<Symbol!("my_app"), Nil>>;
        // A capitalized name, a primitive, and any other type stay the type.
        let _: PhantomData<Path!(@ErrorRaiserComponent)> =
            PhantomData::<PathCons<ErrorRaiserComponent, Nil>>;
        let _: PhantomData<Path!(@u32.bool.str)> =
            PhantomData::<PathCons<u32, PathCons<bool, PathCons<str, Nil>>>>;
        let _: PhantomData<Path!(@app.Vec<u8>)> =
            PhantomData::<PathCons<Symbol!("app"), PathCons<Vec<u8>, Nil>>>;
        let _: PhantomData<Path!(@some_mod::Marker.&'static str)> =
            PhantomData::<PathCons<some_mod::Marker, PathCons<&'static str, Nil>>>;
        let _: PhantomData<Path!(@my_app.ShowImplComponent)> =
            PhantomData::<PathCons<Symbol!("my_app"), PathCons<ShowImplComponent, Nil>>>;
    }
}

/// ## Examples
///
/// The named route, the namespace redirect, the prefix registration, and the wiring key, joined
/// into one program so the route resolves. The page names `FooProviderComponent`,
/// `MyFooComponent`, `AppNamespace`, `ShowWithDebug`, and `MyApp` without declaring them here.
pub mod examples {
    use cgp::core::error::ErrorRaiserComponent;
    use cgp::prelude::*;

    pub type ErrorRoute = Path!(@app.error.ErrorRaiserComponent);

    #[test]
    fn the_named_route_is_a_path_list() {
        let _: PhantomData<ErrorRoute> = PhantomData::<
            PathCons<Symbol!("app"), PathCons<Symbol!("error"), PathCons<ErrorRaiserComponent, Nil>>>,
        >;
    }

    #[cgp_component(FooProvider)]
    pub trait Foo {
        fn foo(&self) -> u8;
    }

    pub struct MyFooComponent;

    cgp_namespace! {
        new MyNamespace {
            FooProviderComponent =>
                @MyFooComponent,
        }
    }

    #[cgp_component(ShowImpl)]
    #[prefix(@show in AppNamespace)]
    pub trait CanShow {
        fn show(&self) -> String;
    }

    #[cgp_impl(new ShowWithDebug)]
    #[uses(core::fmt::Debug)]
    impl ShowImpl {
        fn show(&self) -> String {
            format!("{:?}", self)
        }
    }

    cgp_namespace! {
        new AppNamespace {}
    }

    #[derive(Debug)]
    pub struct MyApp;

    delegate_components! {
        MyApp {
            namespace AppNamespace;

            @show.ShowImplComponent: ShowWithDebug,
        }
    }

    check_components! {
        MyApp {
            ShowImplComponent,
        }
    }

    #[test]
    fn the_prefixed_route_resolves() {
        assert_eq!(MyApp.show(), "MyApp");
    }
}

/// ## Under the hood
///
/// The fully expanded symbol segment and the embedded forms' paths, checked against the traits
/// the macros implement: the namespace's redirect `Delegate`, and the prefix route.
pub mod under_the_hood {
    use cgp::prelude::*;

    pub use super::examples::{
        AppNamespace, FooProviderComponent, MyFooComponent, MyNamespace, ShowImplComponent,
    };

    pub struct MyComp;

    #[test]
    fn a_symbol_segment_nests_a_character_list() {
        let _: PhantomData<Path!(@app)> =
            PhantomData::<PathCons<Symbol<3, Chars<'a', Chars<'p', Chars<'p', Nil>>>>, Nil>>;
        let _: PhantomData<Path!(@MyComp)> = PhantomData::<PathCons<MyComp, Nil>>;
        let _: PhantomData<Path!(@u32)> = PhantomData::<PathCons<u32, Nil>>;
    }

    pub struct Table;

    #[test]
    fn the_embedded_forms_build_the_same_list() {
        let _: PhantomData<<FooProviderComponent as MyNamespace<Table>>::Delegate> =
            PhantomData::<RedirectLookup<Table, PathCons<MyFooComponent, Nil>>>;
        let _: PhantomData<<ShowImplComponent as AppNamespace<Table>>::Delegate> = PhantomData::<
            RedirectLookup<Table, PathCons<Symbol!("show"), PathCons<ShowImplComponent, Nil>>>,
        >;
    }
}
