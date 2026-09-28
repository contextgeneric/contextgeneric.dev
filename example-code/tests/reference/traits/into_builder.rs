//! Code from `docs/reference/traits/builder/into_builder.md` — `IntoBuilder`.
//!
//! Pins the Examples program: a complete record turned into a full builder, one field swapped, and
//! the record rebuilt.

/// ## Examples
pub mod examples {
    use cgp::core::field::traits::TakeField;
    use cgp::prelude::*;

    #[derive(Debug, PartialEq, BuildField)]
    pub struct Person {
        pub first_name: String,
        pub last_name: String,
    }

    pub fn with_last_name(person: Person, last_name: String) -> Person {
        let (_old, remainder) = person
            .into_builder()
            .take_field(PhantomData::<Symbol!("last_name")>);

        remainder
            .build_field(PhantomData::<Symbol!("last_name")>, last_name)
            .finalize_build()
    }

    pub fn demo() {
        let person = Person {
            first_name: "Alice".to_owned(),
            last_name: "Anderson".to_owned(),
        };

        assert_eq!(
            with_last_name(person, "Chen".to_owned()),
            Person {
                first_name: "Alice".to_owned(),
                last_name: "Chen".to_owned(),
            }
        );
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
