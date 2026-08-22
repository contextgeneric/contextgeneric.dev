//! Code from `docs/reference/derives/derive_build_field.md` — *`#[derive(BuildField)]`*.
//!
//! The page's *Under the hood* section shows the companion struct and its impls; those are not
//! repeated here, since re-declaring them beside the derive would be a coherence conflict and
//! `cargo cgp expand` is the check on that section. What this file pins is that each of the page's
//! snippets runs, and that the incomplete build it says will not compile does not.

/// ## Overview
///
/// The opening builder chain. The page shows it against `Person`, which it declares under *Using it*.
pub mod what_its_for {
    use cgp::prelude::*;

    #[derive(BuildField)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct Person {
        pub first_name: String,
        pub last_name: String,
    }

    pub fn build(first: String, last: String) -> Person {
        Person::builder()
            .build_field(PhantomData::<Symbol!("first_name")>, first)
            .build_field(PhantomData::<Symbol!("last_name")>, last)
            .finalize_build()
    }

    #[test]
    fn test_build() {
        assert_eq!(
            build("Alice".to_owned(), "Anderson".to_owned()),
            Person {
                first_name: "Alice".to_owned(),
                last_name: "Anderson".to_owned(),
            }
        );
    }
}

/// The page's central claim, as a rejected snippet: dropping a `build_field` line means
/// `finalize_build` does not resolve, because its impl exists only at the all-present configuration.
///
/// ```compile_fail
/// use cgp::prelude::*;
///
/// #[derive(BuildField)]
/// pub struct Person {
///     pub first_name: String,
///     pub last_name: String,
/// }
///
/// let person: Person = Person::builder()
///     .build_field(PhantomData::<Symbol!("first_name")>, "Alice".to_owned())
///     .finalize_build();
/// ```
pub mod rejected_incomplete_build {}

/// The page's other rejected case: a field's accessor on the companion is in scope only once that
/// field is present, so reading an unset one does not compile. The error arrives as a type mismatch
/// on the tag rather than as a missing method, because the only applicable `HasField` impl is the one
/// for the field that *is* present.
///
/// ```compile_fail
/// use cgp::prelude::*;
///
/// #[derive(BuildField)]
/// pub struct Person {
///     pub first_name: String,
///     pub last_name: String,
/// }
///
/// let partial = Person::builder()
///     .build_field(PhantomData::<Symbol!("first_name")>, "Alice".to_owned());
///
/// let _ = partial.get_field(PhantomData::<Symbol!("last_name")>);
/// ```
pub mod rejected_read_of_an_unset_field {}

/// The page's third rejected case: `build_from` walks the *source's* field list, so a source deriving
/// only `BuildField` cannot be merged into anything. The error is an unsatisfied `HasFields` bound on
/// the source type.
///
/// ```compile_fail
/// use cgp::core::field::impls::CanBuildFrom;
/// use cgp::prelude::*;
///
/// #[derive(BuildField)]
/// pub struct FooBar {
///     pub foo: u64,
///     pub bar: String,
/// }
///
/// #[derive(BuildField)]
/// pub struct FooBarBaz {
///     pub foo: u64,
///     pub bar: String,
///     pub baz: bool,
/// }
///
/// let source = FooBar { foo: 1, bar: "bar".to_owned() };
///
/// let _ = FooBarBaz::builder()
///     .build_from(source)
///     .build_field(PhantomData::<Symbol!("baz")>, true)
///     .finalize_build();
/// ```
pub mod rejected_build_from_without_has_fields {}

/// ## Usage
///
/// The shapes the page lists: a named-field struct, a tuple struct keyed by position, and the
/// fieldless struct whose `builder()` is immediately finalizable.
pub mod using_it {
    use cgp::prelude::*;

    #[derive(BuildField)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct Person {
        pub first_name: String,
        pub last_name: String,
    }

    #[derive(BuildField)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct Pair(pub u32, pub u32);

    #[derive(BuildField)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct NoConfig {}

    #[derive(BuildField)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct Generic<T> {
        pub value: T,
    }

    #[test]
    fn test_a_tuple_struct_is_keyed_by_position() {
        let pair: Pair = Pair::builder()
            .build_field(PhantomData::<Index<0>>, 3)
            .build_field(PhantomData::<Index<1>>, 4)
            .finalize_build();

        assert_eq!(pair, Pair(3, 4));
    }

    #[test]
    fn test_a_fieldless_struct_finalizes_immediately() {
        let config: NoConfig = NoConfig::builder().finalize_build();

        assert_eq!(config, NoConfig {});
    }

    #[test]
    fn test_generics_are_carried_onto_the_companion() {
        let generic: Generic<u32> = Generic::builder()
            .build_field(PhantomData::<Symbol!("value")>, 42)
            .finalize_build();

        assert_eq!(generic, Generic { value: 42 });
    }
}

/// ## Examples
///
/// The three snippets the page shows in order: extending one record into a larger one, reading a set
/// field back out mid-build, and taking a field out and putting it back.
pub mod examples {
    use cgp::core::field::impls::CanBuildFrom;
    use cgp::prelude::*;

    // `build_from` recurses over the *source's* field list, so the source needs `HasFields` as
    // well as the builder. The page says so under *The three ways to fill a field*.
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
        let foo_bar = FooBar {
            foo: 1,
            bar: "bar".to_owned(),
        };

        assert_eq!(
            extend(foo_bar),
            FooBarBaz {
                foo: 1,
                bar: "bar".to_owned(),
                baz: true,
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
}
