//! Code from `docs/reference/providers/with_field.md` — *`WithField`*.
//!
//! Pins that the `WithField<Symbol!("first_name")>` alias wires a `name` getter to read the
//! `first_name` field, the same as the plain `UseField`.

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
            NameGetterComponent: WithField<Symbol!("first_name")>,
        }
    }

    check_components! {
        Person {
            NameGetterComponent,
        }
    }

    #[test]
    fn test_with_field_reads_first_name() {
        let person = Person {
            first_name: "Alice".to_owned(),
        };
        assert_eq!(person.name(), "Alice");
    }
}
