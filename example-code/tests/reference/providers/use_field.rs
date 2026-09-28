//! Code from `docs/reference/providers/use_field.md` — *`UseField`*.
//!
//! Pins the decoupling the page is about: a getter method `name` reads a `first_name` field through
//! `UseField`, the borrowed-view getter from When to use it, and the listings' inputs and
//! type-provider forms from Under the hood. The multi-method getter from Common Mistakes is a
//! trybuild fixture; the `WithField` alias is covered by `with_field.rs`.

/// ## Examples
pub mod examples {
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

    pub fn demo() {
        let person = Person { first_name: "Alice".to_owned() };
        assert_eq!(person.name(), "Alice"); // reads the first_name field
    }

    #[test]
    fn test_demo() {
        demo();
    }
}

/// ## When to use it
///
/// The borrowed-view getters `UseField` already handles: `-> &[u8]` over a `Vec<u8>` field.
pub mod when_to_use_it {
    use cgp::prelude::*;

    #[cgp_getter]
    pub trait HasPayload {
        fn payload(&self) -> &[u8];
    }

    #[derive(HasField)]
    pub struct Message {
        pub body: Vec<u8>,
    }

    delegate_components! {
        Message {
            PayloadGetterComponent: UseField<Symbol!("body")>,
        }
    }

    check_components! {
        Message {
            PayloadGetterComponent,
        }
    }

    #[test]
    fn a_slice_getter_reads_a_vec_field() {
        let message = Message { body: vec![1, 2, 3] };
        assert_eq!(message.payload(), &[1, 2, 3]);
    }
}

/// ## Under the hood
///
/// The getter whose generated `UseField` impl the page lists, for `cargo cgp expand`.
pub mod under_the_hood {
    use cgp::prelude::*;

    #[cgp_getter]
    pub trait HasName {
        fn name(&self) -> &str;
    }

    /// `UseField`'s `TypeProvider` impl: the built-in `TypeProviderComponent` takes it directly, and a
    /// `#[cgp_type]` component takes it through `WithField`.
    pub mod type_provider {
        use core::marker::PhantomData;

        use cgp::core::field::impls::WithField;
        use cgp::core::types::TypeProviderComponent;
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
                TypeProviderComponent: UseField<Symbol!("width")>,
                WidthTypeProviderComponent: WithField<Symbol!("width")>,
            }
        }

        check_components! {
            Rectangle {
                WidthTypeProviderComponent,
            }
        }

        pub fn assert_width_types(
            built_in: PhantomData<<Rectangle as HasType<()>>::Type>,
            named: PhantomData<<Rectangle as HasWidthType>::Width>,
        ) -> (PhantomData<f32>, PhantomData<f32>) {
            (built_in, named)
        }
    }
}
