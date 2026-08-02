//! Code from `docs/reference/traits/can_upcast.md` — *`CanUpcast`*.
//!
//! The page's snippet, run. What this pins beyond "it compiles" is the import path — the trait is
//! not in the prelude — and the totality claim: an upcast into a wider enum always succeeds, so
//! `upcast` returns the target directly rather than a `Result`.

/// ## Examples
pub mod examples {
    use cgp::prelude::*;

    #[derive(Debug, Eq, PartialEq, CgpData)]
    pub enum FooBar {
        Foo(u64),
        Bar(String),
    }

    #[derive(Debug, Eq, PartialEq, CgpData)]
    pub enum FooBarBaz {
        Foo(u64),
        Bar(String),
        Baz(bool),
    }

    #[test]
    fn test_upcast_always_succeeds() {
        use cgp::core::field::impls::CanUpcast;
        use core::marker::PhantomData;

        let wide = FooBar::Foo(1).upcast(PhantomData::<FooBarBaz>);

        assert_eq!(wide, FooBarBaz::Foo(1));
    }
}
