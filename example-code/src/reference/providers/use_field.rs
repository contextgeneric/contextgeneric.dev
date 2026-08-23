//! Code from `docs/reference/providers/use_field.md` — *`UseField`*.
//!
//! Pins the decoupling the page is about: a getter method `name` reads a `first_name` field, once
//! through `UseField` and once through the `WithField` alias. The alias is wired on a second context,
//! since two entries for one component on one context would conflict.

/// ## Examples
pub mod examples {
    use cgp::core::field::impls::WithField;
    use cgp::prelude::*;

    #[cgp_getter]
    pub trait HasName {
        fn name(&self) -> &str;
    }

    #[derive(HasField)]
    pub struct Person {
        pub first_name: String,
    }

    delegate_components! {
        Person {
            NameGetterComponent: UseField<Symbol!("first_name")>,
        }
    }

    check_components! {
        Person {
            NameGetterComponent,
        }
    }

    // The page's `greet` fn.
    fn greet(person: &Person) {
        println!("Hello, {}!", person.name()); // reads the first_name field
    }

    #[test]
    fn test_use_field_reads_first_name() {
        let person = Person {
            first_name: "Alice".to_owned(),
        };
        assert_eq!(person.name(), "Alice");
        greet(&person);
    }

    // The `WithField` alias, on a separate context to avoid a duplicate wiring for one component.
    #[derive(HasField)]
    pub struct PersonWith {
        pub first_name: String,
    }

    delegate_components! {
        PersonWith {
            NameGetterComponent: WithField<Symbol!("first_name")>,
        }
    }

    check_components! {
        PersonWith {
            NameGetterComponent,
        }
    }
}
