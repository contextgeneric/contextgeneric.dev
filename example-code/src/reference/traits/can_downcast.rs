//! Code from `docs/reference/traits/can_downcast.md` — *`CanDowncast`*.
//!
//! The page's snippets, run. What this pins is the import path — the trait is not in the prelude —
//! and the two halves of the page's claim: a downcast succeeds for a shared variant and returns a
//! remainder for one the target lacks.

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
    fn test_downcast_succeeds_for_a_shared_variant() {
        use cgp::core::field::impls::CanDowncast;
        use core::marker::PhantomData;

        assert_eq!(
            FooBarBaz::Bar("hi".to_owned())
                .downcast(PhantomData::<FooBar>)
                .ok(),
            Some(FooBar::Bar("hi".to_owned())),
        );
    }

    #[test]
    fn test_downcast_fails_for_a_variant_the_target_lacks() {
        use cgp::core::field::impls::CanDowncast;
        use core::marker::PhantomData;

        assert_eq!(
            FooBarBaz::Baz(true).downcast(PhantomData::<FooBar>).ok(),
            None,
        );
    }
}
