//! Code from `docs/reference/types/phantom_data.md` — *`PhantomData`*.
//!
//! `PhantomData` is a standard-library type, so there is nothing of CGP's to re-declare. What the
//! page's snippets check are its two roles: declaring a zero-sized struct generic over a type it
//! stores no value of, and passing a type to a call site as a zero-sized value. The Examples program
//! does both through one generic provider. The rejected forms (an unused type parameter, an unused
//! lifetime, and a call that leaves the tag to inference) are trybuild fixtures under
//! `tests/compile_fail/reference/types/`.

/// ## Why a marker needs it
///
/// The fixed form of the page's failing snippet: a marker generic over a parameter it stores no value
/// of carries a `PhantomData` over it and stays zero-sized. (The failing form is the compile-fail
/// fixture `tests/compile_fail/reference/types/phantom_data_a_marker_needs_the_field.rs`.)
pub mod why_a_marker_needs_it {
    use cgp::prelude::*;

    pub struct Multiply<Field>(pub PhantomData<Field>);

    #[test]
    fn test_a_marker_struct_is_zero_sized() {
        assert_eq!(core::mem::size_of::<Multiply<u32>>(), 0);
    }
}

/// ## Examples
pub mod examples {
    use cgp::prelude::*;

    #[cgp_component(Greeter)]
    pub trait CanGreet {
        fn greet(&self) -> String;
    }

    // `new` declares `pub struct GreetField<Tag>(pub PhantomData<Tag>);`
    #[cgp_impl(new GreetField<Tag>)]
    #[uses(HasField<Tag, Value = String>)]
    impl<Tag> Greeter {
        fn greet(&self) -> String {
            let name = self.get_field(PhantomData::<Tag>);
            format!("Hello, {name}!")
        }
    }

    #[derive(HasField)]
    pub struct Person {
        pub name: String,
    }

    delegate_components! {
        Person {
            GreeterComponent: GreetField<Symbol!("name")>,
        }
    }

    check_components! {
        Person {
            GreeterComponent,
        }
    }

    // A marker over two parameters holds one `PhantomData` over a tuple.
    pub struct Add<Left, Right>(pub PhantomData<(Left, Right)>);

    pub fn demo() {
        let person = Person {
            name: "World".to_owned(),
        };
        assert_eq!(person.greet(), "Hello, World!");

        // Both markers occupy no space.
        assert_eq!(core::mem::size_of::<GreetField<Symbol!("name")>>(), 0);
        assert_eq!(core::mem::size_of::<Add<u32, String>>(), 0);
    }

    #[test]
    fn test_demo() {
        demo();
    }
}
