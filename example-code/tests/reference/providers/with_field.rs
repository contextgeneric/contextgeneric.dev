//! Code from `docs/reference/providers/with_field.md` — *`WithField`*.
//!
//! Pins that the `WithField<Symbol!("first_name")>` alias wires a `name` getter to read the
//! `first_name` field, the same as the plain `UseField`, and gives a `#[cgp_type]` component a
//! field's type, which the plain `UseField` cannot.

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

    pub fn demo() {
        let person = Person { first_name: "Alice".to_owned() };
        assert_eq!(person.name(), "Alice");
    }

    #[test]
    fn test_demo() {
        demo();
    }

    /// The type-component form: `WithField` gives a `#[cgp_type]` component a field's type.
    pub mod type_component {
        use core::marker::PhantomData;

        use cgp::core::field::impls::WithField;
        use cgp::prelude::*;

        #[cgp_type]
        pub trait HasWidthType {
            type Width;
        }

        #[derive(HasField)]
        pub struct Rectangle {
            pub width: f32,
        }

        delegate_components! {
            Rectangle {
                WidthTypeProviderComponent: WithField<Symbol!("width")>,
            }
        }

        check_components! {
            Rectangle {
                WidthTypeProviderComponent,
            }
        }

        pub fn width_type(width: PhantomData<<Rectangle as HasWidthType>::Width>) -> PhantomData<f32> {
            width
        }
    }
}
