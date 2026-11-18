//! Code from `docs/reference/macros/blanket_trait.md` — *`#[blanket_trait]`*.
//!
//! The snippets the page rejects live under `tests/compile_fail/reference/macros/`.

/// ## Overview and Usage
///
/// The `FooBar` extension trait, written once and used on a context implementing `Foo` and `Bar`.
/// The page names `Foo` and `Bar` in the Overview and Usage without declaring them until Examples.
pub mod overview_and_usage {
    use cgp::core::macros::blanket_trait;

    pub trait Foo {
        fn foo(&self) -> u32;
    }

    pub trait Bar {
        fn bar(&self) -> u32;
    }

    #[blanket_trait]
    pub trait FooBar: Foo + Bar {
        fn foo_bar(&self) -> u32 {
            self.foo() + self.bar()
        }
    }

    /// The context override, which names the impl's parameter `Ctx`.
    #[blanket_trait(Ctx)]
    pub trait BarFoo: Foo + Bar {
        fn bar_foo(&self) -> u32 {
            self.bar() * self.foo()
        }
    }

    /// Trait generics, which the impl lists before the context parameter.
    #[blanket_trait]
    pub trait Scaled<T: Copy + Into<u32>>: Foo {
        fn scaled(&self, factor: T) -> u32 {
            self.foo() * factor.into()
        }
    }

    /// A trait with no supertraits covers every type.
    #[blanket_trait]
    pub trait Everything {
        const ANSWER: u32 = 42;
    }

    pub struct Ctx;

    impl Foo for Ctx {
        fn foo(&self) -> u32 {
            2
        }
    }

    impl Bar for Ctx {
        fn bar(&self) -> u32 {
            3
        }
    }

    #[test]
    fn the_blanket_impls_apply() {
        assert_eq!(Ctx.foo_bar(), 5);
        assert_eq!(Ctx.bar_foo(), 6);
        assert_eq!(Ctx.scaled(4u8), 8);
        assert_eq!(<String as Everything>::ANSWER, 42);
    }
}

/// ### Lifting an associated type out of a supertrait
///
/// The page names `HasFooTypeAt` and `BarTag` without declaring them. `HasFooTypeAt` is an
/// abstract-type trait keyed by a tag here, implemented directly on the context.
pub mod lifting_an_associated_type_out_of_a_supertrait {
    use cgp::core::macros::blanket_trait;

    pub trait HasFooTypeAt<Tag> {
        type Foo;
    }

    pub struct BarTag;

    #[blanket_trait]
    pub trait HasFooTypeAtBar: HasFooTypeAt<BarTag, Foo = Self::FooBar> {
        type FooBar: Clone;

        /// A method can name the lifted type too; the macro rewrites `Self::FooBar` to the impl's
        /// parameter.
        fn clone_foo(value: &Self::FooBar) -> Self::FooBar {
            value.clone()
        }
    }

    pub struct App;

    impl HasFooTypeAt<BarTag> for App {
        type Foo = String;
    }

    #[test]
    fn the_supertrait_type_is_reexported() {
        let foo: <App as HasFooTypeAtBar>::FooBar = "foo".to_owned();
        assert_eq!(App::clone_foo(&foo), "foo");
    }
}

/// ## Examples
///
/// The page's extension trait and context, and the empty-body trait alias.
pub mod examples {
    use cgp::core::macros::blanket_trait;

    pub trait Foo {
        fn foo(&self);
    }

    pub trait Bar {
        fn bar(&self);
    }

    #[blanket_trait]
    pub trait FooBar: Foo + Bar {
        fn foo_bar(&self) {
            self.foo();
            self.bar();
        }
    }

    pub struct Ctx;

    impl Foo for Ctx {
        fn foo(&self) {}
    }

    impl Bar for Ctx {
        fn bar(&self) {}
    }

    pub fn run(ctx: &Ctx) {
        ctx.foo_bar();
    }

    pub mod trait_alias {
        use cgp::core::macros::blanket_trait;

        use super::{Bar, Ctx, Foo};

        #[blanket_trait]
        pub trait FooBar: Foo + Bar {}

        pub fn needs_both<T: FooBar>(_value: &T) {}

        #[test]
        fn any_foo_and_bar_is_foobar() {
            needs_both(&Ctx);
        }
    }

    #[test]
    fn the_method_runs() {
        run(&Ctx);
    }
}
