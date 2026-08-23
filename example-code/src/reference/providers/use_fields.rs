//! Code from `docs/reference/providers/use_fields.md` — *`UseFields`*.
//!
//! Pins the method-name-equals-field-name convention: wiring a getter to `UseFields` reads the
//! same-named context field.

/// ## Examples
pub mod examples {
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

    fn describe(app: &App) -> &str {
        app.foo() // reads the `foo` field
    }

    #[test]
    fn test_use_fields_reads_same_named_field() {
        let app = App {
            foo: "hi".to_owned(),
        };
        assert_eq!(describe(&app), "hi");
    }
}
