//! Code from `docs/reference/traits/casting/can_downcast.md` — `CanDowncast`.
//!
//! Pins the Examples program: a downcast succeeding for a shared variant and handing back a
//! remainder for one the target lacks. A target variant the source lacks, and a second `downcast`
//! on a remainder, are trybuild fixtures.

/// ## Examples
pub mod examples {
    use cgp::core::field::impls::CanDowncast;
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

    pub fn demo() {
        assert_eq!(
            FooBarBaz::Bar("hi".to_owned())
                .downcast(PhantomData::<FooBar>)
                .ok(),
            Some(FooBar::Bar("hi".to_owned())),
        );

        assert_eq!(FooBarBaz::Baz(true).downcast(PhantomData::<FooBar>).ok(), None);
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
