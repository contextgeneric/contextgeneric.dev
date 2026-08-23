//! Code from `docs/reference/traits/can_build_from.md` — *`CanBuildFrom`*.
//!
//! The page's snippets, run. Two claims are pinned beyond "it compiles": the import path, since the
//! trait is not in the prelude, and that `build_from` returns a *builder* rather than the target, so
//! a second merge and an explicit `build_field` can follow before `finalize_build`.

/// ## Examples
///
/// One struct assembled from two smaller ones.
pub mod examples_two_sources {
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

    #[test]
    fn test_build_from_two_sources() {
        use cgp::core::field::impls::CanBuildFrom;

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
    }
}

/// ## Examples
///
/// A merge mixed with an explicit field, which is the shape the page shows second.
pub mod examples_merge_and_set {
    use cgp::prelude::*;

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

    #[test]
    fn test_merge_then_set() {
        use cgp::core::field::impls::CanBuildFrom;

        let person = Person {
            first_name: "Alice".to_owned(),
            last_name: "Anderson".to_owned(),
        };

        let employee: Employee = Employee::builder()
            .build_from(person)
            .build_field(PhantomData::<Symbol!("employee_id")>, 7)
            .finalize_build();

        assert_eq!(
            employee,
            Employee {
                employee_id: 7,
                first_name: "Alice".to_owned(),
                last_name: "Anderson".to_owned(),
            }
        );
    }
}
