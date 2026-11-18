//! Code from `docs/reference/traits/casting/can_downcast_fields.md` — `CanDowncastFields`.
//!
//! Pins the Examples program: a value tried against three candidate targets in turn, the last step
//! closed by `finalize_extract_result` because the candidates cover every variant.

/// ## Examples
pub mod examples {
    use cgp::core::field::impls::{CanDowncast, CanDowncastFields};
    use cgp::core::field::traits::FinalizeExtractResult;
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

    #[derive(Debug, Eq, PartialEq, CgpData)]
    pub enum JustBaz {
        Baz(bool),
    }

    pub fn classify(value: FooBarBaz) -> &'static str {
        match value.downcast(PhantomData::<JustFoo>) {
            Ok(_) => "foo",
            // `remainder` can no longer be a `Foo`.
            Err(remainder) => match remainder.downcast_fields(PhantomData::<JustBar>) {
                Ok(_) => "bar",
                Err(remainder) => {
                    // Every variant is now covered, so the last attempt cannot fail.
                    let JustBaz::Baz(_) = remainder
                        .downcast_fields(PhantomData::<JustBaz>)
                        .finalize_extract_result();
                    "baz"
                }
            },
        }
    }

    pub fn demo() {
        assert_eq!(classify(FooBarBaz::Foo(1)), "foo");
        assert_eq!(classify(FooBarBaz::Bar("hi".to_owned())), "bar");
        assert_eq!(classify(FooBarBaz::Baz(true)), "baz");
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
