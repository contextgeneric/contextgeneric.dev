//! Code from `docs/reference/traits/builder/build_field.md` — `BuildField`.
//!
//! Pins the Examples program: a record built one field at a time in either order, and a wider record
//! built from a narrower one plus one field. Setting a field twice is a trybuild fixture.

/// ## Examples
pub mod examples {
    use cgp::core::field::impls::CanBuildFrom;
    use cgp::prelude::*;

    #[derive(Debug, PartialEq, HasFields, BuildField)]
    pub struct Person {
        pub first_name: String,
        pub last_name: String,
    }

    #[derive(Debug, PartialEq, BuildField)]
    pub struct Employee {
        pub first_name: String,
        pub last_name: String,
        pub employee_id: u64,
    }

    pub fn demo() {
        // Order does not matter: `last_name` is set first here.
        let person = Person::builder()
            .build_field(PhantomData::<Symbol!("last_name")>, "Chen".to_owned())
            .build_field(PhantomData::<Symbol!("first_name")>, "Alice".to_owned())
            .finalize_build();

        let employee = Employee::builder()
            .build_from(person)
            .build_field(PhantomData::<Symbol!("employee_id")>, 7)
            .finalize_build();

        assert_eq!(
            employee,
            Employee {
                first_name: "Alice".to_owned(),
                last_name: "Chen".to_owned(),
                employee_id: 7,
            }
        );
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
