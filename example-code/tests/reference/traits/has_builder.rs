//! Code from `docs/reference/traits/has_builder.md` — *`HasBuilder`*.
//!
//! The page's three snippets, run: the `extend` merge, the `into_builder`/`take_field` round trip,
//! and reading a set field back off a partial value. The rejected cases the page states are
//! `compile_fail` doctests below, since a page that says something will not compile should be
//! checked on that rather than trusted.

/// ## Examples
pub mod examples {
    use cgp::core::field::impls::CanBuildFrom;
    use cgp::prelude::*;

    // `build_from` walks the *source's* field list, so the source needs `HasFields` too.
    #[derive(HasFields, BuildField)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct FooBar {
        pub foo: u64,
        pub bar: String,
    }

    #[derive(BuildField)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct FooBarBaz {
        pub foo: u64,
        pub bar: String,
        pub baz: bool,
    }

    #[derive(BuildField)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct Person {
        pub first_name: String,
        pub last_name: String,
    }

    pub fn extend(foo_bar: FooBar) -> FooBarBaz {
        FooBarBaz::builder()
            .build_from(foo_bar)
            .build_field(PhantomData::<Symbol!("baz")>, true)
            .finalize_build()
    }

    #[test]
    fn test_extend() {
        assert_eq!(
            extend(FooBar {
                foo: 1,
                bar: "bar".to_owned()
            }),
            FooBarBaz {
                foo: 1,
                bar: "bar".to_owned(),
                baz: true,
            }
        );
    }

    #[test]
    fn test_take_a_field_out_and_put_it_back() {
        // `TakeField` is the one trait in the family that is not in the prelude.
        use cgp::core::field::traits::TakeField;

        let person = Person {
            first_name: "Alice".to_owned(),
            last_name: "Anderson".to_owned(),
        };

        let builder = person.into_builder();

        let (first_name, remainder) = builder.take_field(PhantomData::<Symbol!("first_name")>);
        assert_eq!(first_name, "Alice");

        let person = remainder
            .build_field(PhantomData::<Symbol!("first_name")>, first_name)
            .finalize_build();

        assert_eq!(
            person,
            Person {
                first_name: "Alice".to_owned(),
                last_name: "Anderson".to_owned(),
            }
        );
    }

    #[test]
    fn test_read_a_set_field_mid_build() {
        let partial =
            Person::builder().build_field(PhantomData::<Symbol!("first_name")>, "Alice".to_owned());

        assert_eq!(
            partial.get_field(PhantomData::<Symbol!("first_name")>),
            "Alice"
        );
    }
}

/// The page's central claim: `finalize_build` exists only at the all-present configuration, so an
/// incomplete build does not resolve.
///
/// Rejected snippet — trybuild fixture `tests/compile_fail/reference/traits/has_builder_rejected_incomplete_build.rs`.
pub mod rejected_incomplete_build {}

/// The page's other claim: a `build_from` source deriving only `BuildField` cannot be merged, because
/// the recursion needs the source's `HasFields` shape.
///
/// Rejected snippet — trybuild fixture `tests/compile_fail/reference/traits/has_builder_rejected_build_from_without_has_fields.rs`.
pub mod rejected_build_from_without_has_fields {}
