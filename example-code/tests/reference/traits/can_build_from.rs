//! Code from `docs/reference/traits/casting/can_build_from.md` — `CanBuildFrom`.
//!
//! Pins the Examples program: one struct merged from two sources, and a merge mixed with an explicit
//! field. A source field the target lacks, and two sources sharing a field, are trybuild fixtures.

/// ## Examples
pub mod examples {
    use cgp::core::field::impls::CanBuildFrom;
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

    // A source rarely covers everything, so a merge is often followed by an explicit field.
    #[derive(CgpData)]
    pub struct Person {
        pub first_name: String,
        pub last_name: String,
    }

    #[derive(Debug, Eq, PartialEq, CgpData)]
    pub struct Employee {
        pub employee_id: u64,
        pub first_name: String,
        pub last_name: String,
    }

    pub fn hire(person: Person, employee_id: u64) -> Employee {
        Employee::builder()
            .build_from(person)
            .build_field(PhantomData::<Symbol!("employee_id")>, employee_id)
            .finalize_build()
    }

    pub fn demo() {
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

        let person = Person {
            first_name: "Alice".to_owned(),
            last_name: "Anderson".to_owned(),
        };
        assert_eq!(hire(person, 7).employee_id, 7);
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
