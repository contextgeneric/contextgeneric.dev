//! Code from `docs/reference/types/phantom_data.md` — *`PhantomData`*.
//!
//! `PhantomData` is a standard-library type, so there is nothing of CGP's to re-declare. What the
//! page's snippets check are its two roles: declaring a zero-sized struct generic over a type it
//! stores no value of, and passing a type to a call site as a token. The greet provider exercises the
//! second through `get_field(PhantomData::<Symbol!(...)>)`.

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
///
/// Passing a type to a call site: `PhantomData::<Symbol!("name")>` selects which field the getter
/// reads. The component and the context are filled in around the page's `#[cgp_impl]` snippet so the
/// call resolves. A type carrying two parameters uses one `PhantomData` over a tuple.
pub mod examples {
    use cgp::prelude::*;

    #[cgp_component(Greeter)]
    pub trait CanGreet {
        fn greet(&self);
    }

    #[cgp_impl(new GreetHello)]
    impl Greeter
    where
        Self: HasField<Symbol!("name"), Value = String>,
    {
        fn greet(&self) {
            let name = self.get_field(PhantomData::<Symbol!("name")>);
            println!("Hello, {name}!");
        }
    }

    #[derive(HasField)]
    pub struct App {
        pub name: String,
    }

    delegate_components! {
        App {
            GreeterComponent: GreetHello,
        }
    }

    check_components! {
        App {
            GreeterComponent
        }
    }

    // A type that must carry two parameters uses one `PhantomData` over a tuple.
    pub struct Add<Left, Right>(pub PhantomData<(Left, Right)>);

    #[test]
    fn test_greet_reads_the_name_field() {
        let app = App {
            name: "World".to_owned(),
        };
        app.greet();
    }

    #[test]
    fn test_a_two_parameter_marker_is_zero_sized() {
        assert_eq!(core::mem::size_of::<Add<u32, String>>(), 0);
    }
}
