//! Code from `docs/reference/traits/builder/has_builder.md` — `HasBuilder`.
//!
//! Pins the Examples program: a record extended from a narrower one through `build_from`, and a set
//! field read back off a partial value mid-build. An incomplete build, and a `build_from` source
//! without `HasFields`, are trybuild fixtures.

/// ## Examples
pub mod examples {
    use cgp::core::field::impls::CanBuildFrom;
    use cgp::prelude::*;

    // `build_from` walks the *source's* field list, so the source needs `HasFields` too.
    #[derive(HasFields, BuildField)]
    pub struct FooBar {
        pub foo: u64,
        pub bar: String,
    }

    #[derive(Debug, PartialEq, BuildField)]
    pub struct FooBarBaz {
        pub foo: u64,
        pub bar: String,
        pub baz: bool,
    }

    pub fn extend(foo_bar: FooBar) -> FooBarBaz {
        FooBarBaz::builder()
            .build_from(foo_bar)
            .build_field(PhantomData::<Symbol!("baz")>, true)
            .finalize_build()
    }

    pub fn demo() {
        // A set field can be read back off a still-incomplete builder.
        let partial = FooBarBaz::builder().build_field(PhantomData::<Symbol!("baz")>, true);
        assert!(*partial.get_field(PhantomData::<Symbol!("baz")>));

        let foo_bar_baz = extend(FooBar {
            foo: 1,
            bar: "bar".to_owned(),
        });
        assert_eq!(
            foo_bar_baz,
            FooBarBaz {
                foo: 1,
                bar: "bar".to_owned(),
                baz: true,
            }
        );
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
