//! Code from `docs/reference/providers/with_provider.md` — *`WithProvider`*.
//!
//! Pins that the `WithField` alias — `WithProvider<UseField<...>>` — wires a getter to a named field
//! by adapting the foundational field getter, the When to use it claims about `WithField` on a type
//! component and `WithContext`, and the listings' inputs from Under the hood.

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
        let person = Person { first_name: "Ada".to_owned() };
        assert_eq!(person.name(), "Ada");
    }

    #[test]
    fn test_demo() {
        demo();
    }
}

/// ## When to use it
///
/// `WithField` gives a `#[cgp_type]` component a field's type, which the bare `UseField` cannot, and
/// `WithContext` reads the context's `HasType` entry keyed by the component.
pub mod when_to_use_it {
    use core::marker::PhantomData;

    use cgp::core::field::impls::WithField;
    use cgp::core::types::TypeProviderComponent;
    use cgp::prelude::*;

    #[cgp_type]
    pub trait HasWidthType {
        type Width;
    }

    #[cgp_type]
    pub trait HasNameType {
        type Name;
    }

    #[derive(HasField)]
    pub struct Rectangle {
        pub width: f32,
    }

    delegate_components! {
        Rectangle {
            TypeProviderComponent: UseType<String>,
            WidthTypeProviderComponent: WithField<Symbol!("width")>,
            NameTypeProviderComponent: WithContext,
        }
    }

    check_components! {
        Rectangle {
            WidthTypeProviderComponent,
            NameTypeProviderComponent,
        }
    }

    pub fn assert_types(
        width: PhantomData<<Rectangle as HasWidthType>::Width>,
        name: PhantomData<<Rectangle as HasNameType>::Name>,
    ) -> (PhantomData<f32>, PhantomData<String>) {
        (width, name)
    }
}

/// ## Under the hood
///
/// The type and getter components whose `WithProvider` impls the page lists, for `cargo cgp expand`.
pub mod under_the_hood {
    use cgp::prelude::*;

    #[cgp_type]
    pub trait HasNameType {
        type Name;
    }

    #[cgp_getter]
    pub trait HasName {
        fn name(&self) -> &str;
    }
}
