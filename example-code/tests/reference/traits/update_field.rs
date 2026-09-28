//! Code from `docs/reference/traits/builder/update_field.md` — `UpdateField`.
//!
//! Pins the Examples program: the two transitions `BuildField` and `TakeField` pin, each written as
//! a direct `update_field` call with the target marker named. Leaving the marker to inference, and
//! a field the record does not declare, are trybuild fixtures.

/// ## Examples
pub mod examples {
    use cgp::prelude::*;

    #[derive(Debug, PartialEq, BuildField)]
    pub struct Person {
        pub first_name: String,
        pub last_name: String,
    }

    pub fn demo() {
        // IsNothing -> IsPresent: the old storage is `()`.
        let (old, partial) = UpdateField::<Symbol!("first_name"), IsPresent>::update_field(
            Person::builder(),
            PhantomData,
            "Alice".to_owned(),
        );
        assert_eq!(old, ());

        // IsPresent -> IsNothing: the old storage is the value.
        let (taken, partial) =
            UpdateField::<Symbol!("first_name"), IsNothing>::update_field(partial, PhantomData, ());
        assert_eq!(taken, "Alice");

        let person = partial
            .build_field(PhantomData::<Symbol!("first_name")>, "Bob".to_owned())
            .build_field(PhantomData::<Symbol!("last_name")>, "Chen".to_owned())
            .finalize_build();
        assert_eq!(person.first_name, "Bob");
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
