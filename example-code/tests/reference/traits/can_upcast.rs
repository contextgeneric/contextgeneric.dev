//! Code from `docs/reference/traits/casting/can_upcast.md` — `CanUpcast`.
//!
//! Pins the Examples program: a narrow enum widened into a wider one, with each side deriving only
//! what the cast needs. A target that lacks a source variant is a trybuild fixture.

/// ## Examples
pub mod examples {
    use cgp::core::field::impls::CanUpcast;
    use cgp::prelude::*;

    // The source is walked and taken apart, so it needs its shape and its extractor.
    #[derive(Debug, Eq, PartialEq, HasFields, ExtractField)]
    pub enum FooBar {
        Foo(u64),
        Bar(String),
    }

    // The target is only built into, one variant at a time.
    #[derive(Debug, Eq, PartialEq, FromVariant)]
    pub enum FooBarBaz {
        Foo(u64),
        Bar(String),
        Baz(bool),
    }

    pub fn demo() {
        let wide = FooBar::Foo(1).upcast(PhantomData::<FooBarBaz>);
        assert_eq!(wide, FooBarBaz::Foo(1));

        let wide = FooBar::Bar("hi".to_owned()).upcast(PhantomData::<FooBarBaz>);
        assert_eq!(wide, FooBarBaz::Bar("hi".to_owned()));
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
