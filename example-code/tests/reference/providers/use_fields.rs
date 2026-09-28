//! Code from `docs/reference/providers/use_fields.md` — *`UseFields`*.
//!
//! Pins the method-name-equals-field-name convention: the one-method wiring the Usage section shows,
//! and the two-method getter from Examples, whose `UseFields` impl is the listing in Under the hood.

/// ## Usage
pub mod usage {
    use cgp::prelude::*;

    #[cgp_getter]
    pub trait HasFoo {
        fn foo(&self) -> &str;
    }

    #[derive(HasField)]
    pub struct App {
        pub foo: String,
    }

    delegate_components! {
        App {
            FooGetterComponent: UseFields,
        }
    }

    check_components! {
        App {
            FooGetterComponent,
        }
    }

    #[test]
    fn the_method_reads_the_same_named_field() {
        let app = App { foo: "hi".to_owned() };
        assert_eq!(app.foo(), "hi");
    }
}

/// ## Examples
pub mod examples {
    use cgp::prelude::*;

    #[cgp_getter]
    pub trait HasFooBar {
        fn foo(&self) -> &str;
        fn bar(&self) -> &u8;
    }

    #[derive(HasField)]
    pub struct App {
        pub foo: String,
        pub bar: u8,
    }

    delegate_components! {
        App {
            FooBarGetterComponent: UseFields,
        }
    }

    check_components! {
        App {
            FooBarGetterComponent,
        }
    }

    pub fn demo() {
        let app = App { foo: "hi".to_owned(), bar: 7 };

        assert_eq!(app.foo(), "hi"); // reads the `foo` field
        assert_eq!(*app.bar(), 7); // reads the `bar` field
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
