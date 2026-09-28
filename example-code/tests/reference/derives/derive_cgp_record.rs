//! Code from `docs/reference/derives/derive_cgp_record.md` — *`#[derive(CgpRecord)]`*.
//!
//! The page's worked example, run. Written with `CgpRecord` rather than the umbrella, which is the
//! page's own claim that the two emit the same output on a struct.

/// ## Usage
///
/// The struct shapes the page lists: a raw-identifier field, a tuple struct keyed by position, a
/// fieldless struct whose builder finalizes at once, and a generic struct.
pub mod usage {
    use cgp::prelude::*;

    #[derive(CgpRecord, Debug, Eq, PartialEq)]
    pub struct Entry {
        pub r#type: String,
    }

    #[derive(CgpRecord, Debug, Eq, PartialEq)]
    pub struct Pair(pub u32, pub u32);

    #[derive(CgpRecord, Debug, Eq, PartialEq)]
    pub struct NoConfig {}

    #[derive(CgpRecord, Debug, Eq, PartialEq)]
    pub struct Generic<T> {
        pub value: T,
    }

    #[test]
    fn test_every_shape_builds() {
        let entry: Entry = Entry::builder()
            .build_field(PhantomData::<Symbol!("type")>, "text".to_owned())
            .finalize_build();
        assert_eq!(entry.get_field(PhantomData::<Symbol!("type")>), "text");

        let pair: Pair = Pair::builder()
            .build_field(PhantomData::<Index<0>>, 3)
            .build_field(PhantomData::<Index<1>>, 4)
            .finalize_build();
        assert_eq!(pair, Pair(3, 4));

        let config: NoConfig = NoConfig::builder().finalize_build();
        assert_eq!(config, NoConfig {});

        let generic: Generic<u8> = Generic::builder()
            .build_field(PhantomData::<Symbol!("value")>, 1)
            .finalize_build();
        assert_eq!(generic, Generic { value: 1 });
    }
}

/// ## Examples
///
/// The record half: `promote`, which merges one record into another's builder.
pub mod examples {
    use cgp::core::field::impls::CanBuildFrom;
    use cgp::prelude::*;

    #[derive(CgpRecord, Debug, Eq, PartialEq)]
    pub struct Person {
        pub first_name: String,
        pub last_name: String,
    }

    #[derive(CgpRecord, Debug, Eq, PartialEq)]
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

/// ## Common Mistakes
///
/// A single-field tuple struct's representation is the inner type, not a one-element product.
pub mod common_mistakes {
    use cgp::prelude::*;

    #[derive(CgpRecord)]
    pub struct Wrapper(pub String);

    pub fn wrapper_shape(fields: <Wrapper as HasFields>::Fields) -> String {
        fields
    }
}
