//! Compiled counterparts of the `Name` sections of `blog/2026-11-25-incoherent-rust-today.md`.
//! Modules follow the post's headings. The post's other CGP snippets are in `example-code`'s
//! `tests/blog/incoherent_rust_today.rs`; the struct-binding type error is a compile-fail fixture
//! under `tests/compile_fail/blog/`.
#![allow(dead_code, unused_variables)]

mod using_incoherent_trait_bounds {
    use cgp::prelude::*;

    #[cgp_component(NameImpl)]
    pub trait Name<T> {
        const NAME: &'static str;
    }

    #[cgp_impl(new DummyName)]
    impl<T> NameImpl<T> {
        const NAME: &'static str = "dummy";
    }

    #[cgp_impl(new RealName)]
    impl<T> NameImpl<T> {
        const NAME: &'static str = core::any::type_name::<T>();
    }

    pub struct Foo<T>(pub T);

    #[cgp_fn]
    #[uses(Name<T>)]
    pub fn do_stuff<T>(&self, foo: &Foo<T>) {
        println!("{}", Self::NAME);
    }

    #[test]
    fn each_context_names_the_type_its_own_way() {
        let foo = Foo(1);

        {
            struct Context;

            delegate_components! {
                Context {
                    NameImplComponent: DummyName,
                }
            }

            // prints "dummy"
            Context.do_stuff(&foo);
            assert_eq!(<Context as Name<i32>>::NAME, "dummy");
        }

        {
            struct Context;

            delegate_components! {
                Context {
                    NameImplComponent: RealName,
                }
            }

            // prints "i32"
            Context.do_stuff(&foo);
            assert_eq!(<Context as Name<i32>>::NAME, "i32");
        }
    }
}

mod use_of_incoherent_trait_bounds_within_a_struct {
    use cgp::prelude::*;
    use core::marker::PhantomData;

    // `Name`, `DummyName`, and `RealName` as the previous section defines them.
    #[cgp_component(NameImpl)]
    pub trait Name<T> {
        const NAME: &'static str;
    }

    #[cgp_impl(new DummyName)]
    impl<T> NameImpl<T> {
        const NAME: &'static str = "dummy";
    }

    #[cgp_impl(new RealName)]
    impl<T> NameImpl<T> {
        const NAME: &'static str = core::any::type_name::<T>();
    }

    pub struct Foo<Context: ?Sized, T> {
        pub value: T,
        pub phantom: PhantomData<Context>,
    }

    #[cgp_fn]
    #[uses(Name<T>)]
    pub fn do_stuff<T>(&self, foo: &Foo<Self, T>) {
        println!("{}", Self::NAME);
    }

    pub struct ContextA;

    delegate_components! {
        ContextA {
            NameImplComponent: DummyName,
        }
    }

    pub struct ContextB;

    delegate_components! {
        ContextB {
            NameImplComponent: RealName,
        }
    }

    #[test]
    fn the_struct_keeps_its_context() {
        let foo: Foo<ContextA, i32> = Foo {
            value: 1,
            phantom: PhantomData,
        };

        ContextA.do_stuff(&foo);
        // `ContextB.do_stuff(&foo)` is the compile-fail fixture
        // `tests/compile_fail/blog/incoherent_rust_today_struct_context.rs`.
    }
}

mod impl_specific_bindings_within_a_struct {
    use cgp::prelude::*;
    use core::marker::PhantomData;

    // `Name` and `RealName` as the earlier sections define them.
    #[cgp_component(NameImpl)]
    pub trait Name<T> {
        const NAME: &'static str;
    }

    #[cgp_impl(new RealName)]
    impl<T> NameImpl<T> {
        const NAME: &'static str = core::any::type_name::<T>();
    }

    pub struct Foo<NameProvider, T> {
        pub value: T,
        pub phantom: PhantomData<NameProvider>,
    }

    #[cgp_fn]
    #[use_provider(NameProvider: NameImpl<T>)]
    pub fn do_stuff<NameProvider, T>(&self, foo: &Foo<NameProvider, T>) {
        println!("{}", NameProvider::NAME);
    }

    #[test]
    fn any_context_uses_the_struct_s_provider() {
        pub struct ContextA;
        pub struct ContextB;

        let foo: Foo<RealName, i32> = Foo {
            value: 1,
            phantom: PhantomData,
        };

        // Both contexts can call do_stuff on the same foo value,
        // because the Name implementation is fixed by RealName, not by the context.
        ContextA.do_stuff(&foo);
        ContextB.do_stuff(&foo);
    }
}

mod incoherent_functions {
    use cgp::prelude::*;

    // `Name` and `Foo` as the earlier sections define them.
    #[cgp_component(NameImpl)]
    pub trait Name<T> {
        const NAME: &'static str;
    }

    #[cgp_impl(new DummyName)]
    impl<T> NameImpl<T> {
        const NAME: &'static str = "dummy";
    }

    pub struct Foo<T>(pub T);

    #[cgp_fn]
    #[uses(Name<T>)]
    pub fn do_stuff<T>(&self, foo: &Foo<T>) {
        println!("{}", Self::NAME);
    }

    #[cgp_fn]
    #[uses(DoStuff<T>)]
    pub fn do_more_stuff<T>(&self, foo: &Foo<T>) {
        self.do_stuff(foo);
    }

    /// The post's hand-written rendering of what the two functions expand to.
    mod expansion {
        use super::{Foo, Name};

        pub trait DoStuff<T> {
            fn do_stuff(&self, foo: &Foo<T>);
        }

        impl<Context, T> DoStuff<T> for Context
        where
            Context: Name<T>,
        {
            fn do_stuff(&self, foo: &Foo<T>) {
                println!("{}", Self::NAME);
            }
        }

        pub trait DoMoreStuff<T> {
            fn do_more_stuff(&self, foo: &Foo<T>);
        }

        impl<Context, T> DoMoreStuff<T> for Context
        where
            Context: DoStuff<T>,
        {
            fn do_more_stuff(&self, foo: &Foo<T>) {
                self.do_stuff(foo);
            }
        }
    }

    #[test]
    fn the_caller_never_names_the_inner_bound() {
        struct Context;

        delegate_components! {
            Context {
                NameImplComponent: DummyName,
            }
        }

        Context.do_more_stuff(&Foo(1));
    }
}
