//! Code from `docs/reference/macros/cgp_namespace.md` — *`cgp_namespace!`*.

/// ## Overview
///
/// The opening snippet: a namespace lifted out of any one context. The page elides the body as
/// `// shared wiring lives here`; it is empty here, since the shared wiring is the subject of the
/// sections below.
pub mod overview {
    use cgp::prelude::*;

    cgp_namespace! {
        new AppNamespace {
            // The page elides the body. Nothing is wired here.
        }
    }
}

/// ## Usage
///
/// The `new` namespace with one redirect. The page names `FooProviderComponent` and
/// `MyFooComponent` without declaring them, so the component and the path marker are declared
/// here, and a context joins the namespace and binds the redirected path so the route can be
/// checked end to end.
pub mod usage {
    use cgp::prelude::*;

    #[cgp_component(FooProvider)]
    pub trait Foo {
        fn foo(&self) -> String;
    }

    #[cgp_impl(new DummyFoo)]
    impl FooProvider {
        fn foo(&self) -> String {
            "foo".to_owned()
        }
    }

    /// The type the `@MyFooComponent` path segment names.
    pub struct MyFooComponent;

    cgp_namespace! {
        new MyNamespace {
            FooProviderComponent =>
                @MyFooComponent,
        }
    }

    pub struct App;

    delegate_components! {
        App {
            namespace MyNamespace;

            @MyFooComponent: DummyFoo,
        }
    }

    mod check_app {
        use super::*;
        check_components! { App { FooProviderComponent } }
    }

    #[test]
    fn the_redirect_lands_on_the_bound_path() {
        assert_eq!(App.foo(), "foo");
    }
}

/// ### The rest of the body grammar
///
/// The two shared forms the section says are worth writing in a namespace: `open`, which is
/// another spelling of `Component => @Component`, and a nested table, which every joining context
/// inherits. The page names no types here; they are declared.
pub mod the_rest_of_the_body_grammar {
    use cgp::prelude::*;

    #[cgp_component(ShapeArea)]
    #[derive_delegate(UseDelegate<Shape>)]
    pub trait CanShapeArea<Shape> {
        fn shape_area(&self, shape: &Shape) -> f64;
    }

    #[cgp_component(Greeter)]
    pub trait CanGreet {
        fn greet(&self) -> String;
    }

    #[cgp_impl(new AnyArea)]
    impl<Shape> ShapeArea<Shape> {
        fn shape_area(&self, _shape: &Shape) -> f64 {
            1.0
        }
    }

    #[cgp_impl(new GreetHello)]
    impl Greeter {
        fn greet(&self) -> String {
            "hello".to_owned()
        }
    }

    cgp_namespace! {
        new AppNamespace {
            open GreeterComponent;

            ShapeAreaComponent:
                UseDelegate<new ShapeTable {
                    u32: AnyArea,
                    String: AnyArea,
                }>,
        }
    }

    pub struct App;

    delegate_components! {
        App {
            namespace AppNamespace;

            @GreeterComponent: GreetHello,
        }
    }

    mod check_app {
        use super::*;
        check_components! {
            App {
                GreeterComponent,
                ShapeAreaComponent: [u32, String],
            }
        }
    }
}

/// ## Inheriting from a parent
///
/// The child that rewrites the `@cgp.core.error` prefix onto `@app`, and the header-only child
/// with nothing of its own to add. The page does not declare `BaseNamespace`; here it inherits
/// `DefaultNamespace`, which is where CGP's error components register under `@cgp.core.error`, so
/// the rewrite has a subtree to reroute and a joining context can wire the error components under
/// the shorter `@app.*` path.
pub mod inheriting_from_a_parent {
    use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
    use cgp::extra::error::RaiseFrom;
    use cgp::prelude::*;

    cgp_namespace! { new BaseNamespace: DefaultNamespace {} }

    cgp_namespace! {
        new ExtendedNamespace: BaseNamespace {
            @cgp.core.error =>
                @app,
        }
    }

    pub struct App;

    delegate_components! {
        App {
            namespace ExtendedNamespace;

            @app.ErrorTypeProviderComponent: UseType<String>,
            @app.ErrorRaiserComponent.String: RaiseFrom,
        }
    }

    mod check_app {
        use super::*;
        check_components! {
            App {
                ErrorTypeProviderComponent,
                ErrorRaiserComponent: String,
            }
        }
    }

    /// The braceless header. It expands to what `new ExtendedNamespace: BaseNamespace {}` does,
    /// so a context joining it resolves everything `BaseNamespace` does, and the error components
    /// stay under their full `@cgp.core.error.*` path because nothing rewrites it.
    pub mod header_only {
        use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
        use cgp::extra::error::RaiseFrom;
        use cgp::prelude::*;

        use super::BaseNamespace;

        cgp_namespace! {
            new ExtendedNamespace: BaseNamespace
        }

        pub struct App;

        delegate_components! {
            App {
                namespace ExtendedNamespace;

                @cgp.core.error.ErrorTypeProviderComponent: UseType<String>,
                @cgp.core.error.ErrorRaiserComponent.String: RaiseFrom,
            }
        }

        mod check_app {
            use super::*;
            check_components! {
                App {
                    ErrorTypeProviderComponent,
                    ErrorRaiserComponent: String,
                }
            }
        }
    }
}

/// A header-only child written without `new`, which emits only the inheritance impl, so the trait
/// and struct it names are declared by hand.
pub mod header_without_new {
    use cgp::prelude::*;

    cgp_namespace! { new BaseNamespace {} }

    pub trait ExtendedNamespace<__Table__> {
        type Delegate;
    }

    pub struct __ExtendedNamespaceComponents;

    cgp_namespace! { ExtendedNamespace: BaseNamespace }
}

/// ## The other two halves: registering, and joining
///
/// A component registered under a prefix, and a context joining the namespace and binding the
/// prefixed path. `AppNamespace` and `ShowWithDebug` are named by the page without being declared
/// in this section (the provider appears under *Examples*), so both are declared here.
pub mod the_other_two_halves_registering_and_joining {
    use cgp::prelude::*;

    cgp_namespace! { new AppNamespace {} }

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

    #[derive(Debug)]
    pub struct MyApp;

    delegate_components! {
        MyApp {
            namespace AppNamespace;

            @show.ShowImplComponent: ShowWithDebug,
        }
    }

    mod check_my_app {
        use super::*;
        check_components! { MyApp { ShowImplComponent } }
    }
}

/// ## Examples
///
/// The complete program, exactly as the page shows it, followed by the inheritance example.
pub mod examples {
    use cgp::prelude::*;

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
    fn the_context_owns_the_provider() {
        assert_eq!(MyApp.show(), "MyApp");
    }

    /// The inheritance example. The page continues from the program above with the component
    /// registered into the base; a context joining the header-only child inherits the whole chain,
    /// so the component is re-declared here with its prefix in `BaseNamespace`.
    pub mod inheritance {
        use cgp::prelude::*;

        #[cgp_component(ShowImpl)]
        #[prefix(@show in BaseNamespace)]
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

        cgp_namespace! { new BaseNamespace {} }

        cgp_namespace! { new ExtendedNamespace: BaseNamespace }

        #[derive(Debug)]
        pub struct MyApp;

        delegate_components! {
            MyApp {
                namespace ExtendedNamespace;

                @show.ShowImplComponent: ShowWithDebug,
            }
        }

        mod check_my_app {
            use super::*;
            check_components! { MyApp { ShowImplComponent } }
        }

        #[test]
        fn the_child_inherits_the_chain() {
            assert_eq!(MyApp.show(), "MyApp");
        }
    }
}

/// ## When to use it
///
/// An aggregate provider behind a namespace path, in both shapes the section names: a bundle keyed
/// by bare component names, and a bundle grouping paths, which must join the namespace itself.
pub mod when_to_use_it {
    use cgp::prelude::*;

    cgp_namespace! { new AppNamespace {} }

    #[cgp_component(Greeter)]
    #[prefix(@app.core.user in AppNamespace)]
    pub trait CanGreet {
        fn greet(&self) -> u8;
    }

    #[cgp_impl(new GreetOne)]
    impl Greeter {
        fn greet(&self) -> u8 {
            1
        }
    }

    delegate_components! {
        new UserComponents {
            GreeterComponent: GreetOne,
        }
    }

    pub struct App;

    delegate_components! {
        App {
            namespace AppNamespace;

            @app.core.user: UserComponents,
        }
    }

    delegate_components! {
        new CoreComponents {
            namespace AppNamespace;

            @app.core.user: UserComponents,
        }
    }

    pub struct CoreApp;

    delegate_components! {
        CoreApp {
            namespace AppNamespace;

            @app.core: CoreComponents,
        }
    }

    mod check_apps {
        use super::*;
        check_components! { App { GreeterComponent } }
        check_components! { CoreApp { GreeterComponent } }
    }

    #[test]
    fn both_bundles_answer_through_the_path() {
        assert_eq!(App.greet(), 1);
        assert_eq!(CoreApp.greet(), 1);
    }
}

/// ## Under the hood
///
/// The input whose expansion the page walks through. Only the input is checked here; the generated
/// items the page shows are the job of `cargo cgp expand`, which this crate does not replace.
pub mod under_the_hood {
    use cgp::prelude::*;

    #[cgp_component(FooProvider)]
    pub trait Foo {
        fn foo(&self) -> String;
    }

    pub struct MyFooComponent;

    cgp_namespace! {
        new MyNamespace {
            FooProviderComponent =>
                @MyFooComponent,
        }
    }
}

/// ## Common Mistakes
///
/// Every error the page quotes has a trybuild fixture.
///
/// Joining two namespaces on one context, `E0119` —
/// `tests/compile_fail/reference/macros/cgp_namespace_common_mistakes_two_namespaces_joined.rs`.
///
/// Two namespaces inheriting each other, `E0275` —
/// `tests/compile_fail/reference/macros/cgp_namespace_common_mistakes_inheritance_cycle.rs`.
///
/// A namespace inheriting itself, `E0207` —
/// `tests/compile_fail/reference/macros/cgp_namespace_common_mistakes_self_inheriting.rs`.
///
/// A context overriding a key its namespace binds, `E0119` —
/// `tests/compile_fail/reference/macros/cgp_namespace_common_mistakes_context_overrides_bound_key.rs`.
///
/// A child namespace adding a longer path beneath one its parent routes, `E0119` —
/// `tests/compile_fail/reference/macros/cgp_namespace_common_mistakes_child_redefines_parent_path.rs`.
///
/// A registered component whose route nothing binds, `E0277` —
/// `tests/compile_fail/reference/macros/cgp_namespace_common_mistakes_unbound_route.rs`.
///
/// A `namespace` statement written with a path, ``expected `;` `` — shared with the
/// `delegate_components!` page, as
/// `tests/compile_fail/reference/macros/delegate_components_the_namespace_statements_namespace_path.rs`.
///
/// The *Inheriting from a parent* section also states that only the end of the input may follow a
/// header written without braces; the macro's `expected curly braces` rejection of a stray token is
/// `tests/compile_fail/reference/macros/cgp_namespace_inheriting_from_a_parent_stray_token.rs`.
pub mod common_mistakes {}
