//! Code from `docs/reference/providers/with_provider.md` — *`WithProvider`*.
//!
//! Pins that the `WithField` alias — `WithProvider<UseField<...>>` — wires a getter to a named field
//! by adapting the foundational field getter.

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
    fn test_with_field_adapts_use_field() {
        let person = Person {
            first_name: "Ada".to_owned(),
        };
        assert_eq!(person.name(), "Ada");
    }
}
