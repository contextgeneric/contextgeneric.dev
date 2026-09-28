//! Code from `docs/reference/providers/with_context.md` — *`WithContext`*.
//!
//! Pins both forms the page's Usage section describes: a `#[cgp_type]` component resolved through the
//! context's own `HasType`, which is also the Examples program, and a single-method getter read
//! through a hand-written `HasField` impl keyed by the getter's marker.

/// ## Usage
///
/// The getter form: `WithContext` reads the context's `HasField<NameGetterComponent>` entry.
pub mod usage {
    use core::marker::PhantomData;

    use cgp::prelude::*;

    #[cgp_getter]
    pub trait HasName {
        fn name(&self) -> &str;
    }

    pub struct Person {
        pub first_name: String,
    }

    // No derive keys a field by a component marker, so the context implements it by hand.
    impl HasField<NameGetterComponent> for Person {
        type Value = String;

        fn get_field(&self, _tag: PhantomData<NameGetterComponent>) -> &String {
            &self.first_name
        }
    }

    delegate_components! {
        Person {
            NameGetterComponent: WithContext,
        }
    }

    check_components! {
        Person {
            NameGetterComponent,
        }
    }

    #[test]
    fn the_getter_reads_the_marker_keyed_field() {
        let person = Person { first_name: "Ada".to_owned() };
        assert_eq!(person.name(), "Ada");
    }
}

/// ## Examples
pub mod examples {
    use core::marker::PhantomData;

    use cgp::core::types::TypeProviderComponent;
    use cgp::prelude::*;

    #[cgp_type]
    pub trait HasNameType {
        type Name;
    }

    pub struct App;

    delegate_components! {
        App {
            TypeProviderComponent: UseType<String>,
            NameTypeProviderComponent: WithContext,
        }
    }

    check_components! {
        App {
            NameTypeProviderComponent,
        }
    }

    pub fn name_type(name: PhantomData<<App as HasNameType>::Name>) -> PhantomData<String> {
        name
    }
}
