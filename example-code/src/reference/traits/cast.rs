//! Code from `docs/reference/traits/cast.md` — *`CanUpcast`, `CanDowncast` & `CanBuildFrom`*.
//!
//! The page's own snippets, run. What this file pins beyond "it compiles" is the import path —
//! none of the four traits is in the prelude — and the two claims the page makes about totality:
//! an upcast always succeeds, and a downcast against a variant the target lacks returns `Err`.

/// ## Examples
///
/// The two enums the page shows, and both directions of the variant cast.
pub mod examples_variant {
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

/// ## Examples
///
/// The record cast: one struct assembled from two smaller ones.
pub mod examples_record {
    use cgp::prelude::*;

    #[derive(CgpData)]
    pub struct FooBar {
        pub foo: u64,
        pub bar: String,
    }

    #[derive(CgpData)]
    pub struct Baz {
        pub baz: bool,
    }

    #[derive(Debug, Eq, PartialEq, CgpData)]
    pub struct FooBarBaz {
        pub foo: u64,
        pub bar: String,
        pub baz: bool,
    }

    #[test]
    fn test_build_from_two_sources() {
        use cgp::core::field::impls::CanBuildFrom;

        let combined: FooBarBaz = FooBarBaz::builder()
            .build_from(FooBar {
                foo: 1,
                bar: "bar".to_owned(),
            })
            .build_from(Baz { baz: true })
            .finalize_build();

        assert_eq!(
            combined,
            FooBarBaz {
                foo: 1,
                bar: "bar".to_owned(),
                baz: true,
            }
        );
    }
}
