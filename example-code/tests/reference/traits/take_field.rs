//! Code from `docs/reference/traits/builder/take_field.md` — `TakeField`.
//!
//! Pins the Examples program: a field taken out of a full builder, changed, and put back. Taking an
//! absent field is a trybuild fixture.

/// ## Examples
pub mod examples {
    use cgp::core::field::traits::TakeField;
    use cgp::prelude::*;

    #[derive(Debug, PartialEq, BuildField)]
    pub struct Person {
        pub first_name: String,
        pub last_name: String,
    }

    pub fn shout_first_name(person: Person) -> Person {
        let (first_name, remainder) = person
            .into_builder()
            .take_field(PhantomData::<Symbol!("first_name")>);

        remainder
            .build_field(
                PhantomData::<Symbol!("first_name")>,
                first_name.to_uppercase(),
            )
            .finalize_build()
    }

    pub fn demo() {
        let person = Person {
            first_name: "Alice".to_owned(),
            last_name: "Chen".to_owned(),
        };

        assert_eq!(shout_first_name(person).first_name, "ALICE");
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
