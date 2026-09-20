//! Code from `docs/comparisons/row-polymorphism.md` — *Row polymorphism, structural typing, and
//! extensible data types*.

/// ## A struct is a closed row; `HasField` is row containment
pub mod a_struct_is_a_closed_row {
    use cgp::prelude::*;

    #[derive(HasField, HasFields)]
    pub struct Person {
        pub first_name: String,
        pub last_name: String,
    }

    #[cgp_fn]
    pub fn full_name(&self, #[implicit] first_name: &str, #[implicit] last_name: &str) -> String {
        format!("{first_name} {last_name}")
    }

    #[derive(HasField)]
    pub struct Employee {
        pub first_name: String,
        pub last_name: String,
        pub badge: u32,
    }

    #[test]
    fn any_context_with_the_fields_qualifies() {
        let person = Person {
            first_name: "Ada".to_owned(),
            last_name: "Lovelace".to_owned(),
        };
        let employee = Employee {
            first_name: "Ada".to_owned(),
            last_name: "Lovelace".to_owned(),
            badge: 36,
        };
        assert_eq!(person.full_name(), "Ada Lovelace");
        assert_eq!(employee.full_name(), "Ada Lovelace");
    }
}

/// ## Building a record is row combination
pub mod building_a_record_is_row_combination {
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
    fn two_rows_combine_into_one() {
        use cgp::core::field::impls::CanBuildFrom;

        let combined: FooBarBaz = FooBarBaz::builder()
            .build_from(FooBar {
                foo: 1,
                bar: "bar".into(),
            })
            .build_from(Baz { baz: true })
            .finalize_build();

        assert_eq!(
            combined,
            FooBarBaz {
                foo: 1,
                bar: "bar".into(),
                baz: true,
            }
        );
    }
}

/// ## An enum is a row-typed sum; upcast and downcast are injection and branching
pub mod an_enum_is_a_row_typed_sum {
    use cgp::prelude::*;

    #[derive(Debug, Eq, PartialEq, CgpData)]
    pub enum Reading {
        Temperature(u64),
        Label(String),
    }

    #[derive(Debug, Eq, PartialEq, CgpData)]
    pub enum ExtendedReading {
        Temperature(u64),
        Label(String),
        Flag(bool),
    }

    #[test]
    fn upcast_is_injection_and_downcast_is_guarded_projection() {
        use cgp::core::field::impls::{CanDowncast, CanUpcast};

        let wide = Reading::Temperature(21).upcast(PhantomData::<ExtendedReading>);
        assert_eq!(wide, ExtendedReading::Temperature(21));

        let narrow = ExtendedReading::Label("north".to_owned()).downcast(PhantomData::<Reading>);
        assert_eq!(narrow.ok(), Some(Reading::Label("north".to_owned())));

        let narrow = ExtendedReading::Flag(true).downcast(PhantomData::<Reading>);
        assert!(narrow.is_err());
    }
}
