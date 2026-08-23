//! Code from `docs/reference/traits/can_downcast_fields.md` — *`CanDowncastFields`*.
//!
//! The page's chain, run. The load-bearing check is that the second attempt is written with
//! `downcast_fields` rather than `downcast`: a remainder is an extractor rather than an enum, so the
//! two halves of the pair are not interchangeable.

/// ## Examples
pub mod examples {
    use cgp::prelude::*;

    #[derive(Debug, Eq, PartialEq, CgpData)]
    pub enum FooBarBaz {
        Foo(u64),
        Bar(String),
        Baz(bool),
    }

    #[derive(Debug, Eq, PartialEq, CgpData)]
    pub enum JustFoo {
        Foo(u64),
    }

    #[derive(Debug, Eq, PartialEq, CgpData)]
    pub enum JustBar {
        Bar(String),
    }

    #[test]
    fn test_a_chain_of_two_candidate_targets() {
        use cgp::core::field::impls::{CanDowncast, CanDowncastFields};
        use core::marker::PhantomData;

        let value = FooBarBaz::Bar("hi".to_owned());

        match value.downcast(PhantomData::<JustFoo>) {
            Ok(_foo) => panic!("the value is not a Foo"),
            Err(remainder) => {
                let bar = remainder.downcast_fields(PhantomData::<JustBar>);

                assert!(bar.is_ok());
            }
        }
    }
}
