//! Code from `docs/reference/derives/derive_cgp_record.md` — *`#[derive(CgpRecord)]`*.
//!
//! The page's worked example, run. Written with `CgpRecord` rather than the umbrella, which is the
//! page's own claim that the two emit the same output on a struct.

/// ## Examples
///
/// The record half: `promote`, which merges one record into another's builder.
pub mod examples_record {
    use cgp::core::field::impls::CanBuildFrom;
    use cgp::prelude::*;

    #[derive(CgpRecord)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct Person {
        pub first_name: String,
        pub last_name: String,
    }

    #[derive(CgpRecord)]
    #[derive(Debug, Eq, PartialEq)]
    pub struct Employee {
        pub employee_id: u64,
        pub first_name: String,
        pub last_name: String,
    }

    pub fn promote(person: Person, id: u64) -> Employee {
        Employee::builder()
            .build_from(person)
            .build_field(PhantomData::<Symbol!("employee_id")>, id)
            .finalize_build()
    }

    #[test]
    fn test_promote() {
        let person = Person {
            first_name: "Alice".to_owned(),
            last_name: "Anderson".to_owned(),
        };

        assert_eq!(
            promote(person, 1),
            Employee {
                employee_id: 1,
                first_name: "Alice".to_owned(),
                last_name: "Anderson".to_owned(),
            }
        );
    }
}
