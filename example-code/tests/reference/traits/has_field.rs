//! Code from `docs/reference/traits/field-access/has_field.md` — `HasField`.
//!
//! Pins the Examples program: a provider that bounds on a field and reads it, the `#[implicit]` form
//! of the same read, and a read through a `Box` that resolves by the `Deref` forwarding impl. A
//! misspelled field name and a `Deref` type that also derives the field are trybuild fixtures.

/// ## Examples
pub mod examples {
    use cgp::prelude::*;

    #[cgp_component(Greeter)]
    pub trait CanGreet {
        fn greet(&self) -> String;
    }

    #[cgp_impl(new GreetHello)]
    impl Greeter
    where
        Self: HasField<Symbol!("name"), Value = String>,
    {
        fn greet(&self) -> String {
            format!("Hello, {}!", self.get_field(PhantomData::<Symbol!("name")>))
        }
    }

    #[derive(HasField)]
    pub struct Person {
        pub name: String,
    }

    delegate_components! {
        Person {
            GreeterComponent: GreetHello,
        }
    }

    check_components! {
        Person {
            GreeterComponent,
        }
    }

    pub fn read_name<Context>(context: &Context) -> &String
    where
        Context: HasField<Symbol!("name"), Value = String>,
    {
        context.get_field(PhantomData)
    }

    pub fn demo() {
        let person = Person {
            name: "Ada".to_owned(),
        };
        assert_eq!(person.greet(), "Hello, Ada!");

        // `Box<Person>` has the field through the `Deref` forwarding impl.
        let boxed = Box::new(Person {
            name: "Alice".to_owned(),
        });
        assert_eq!(read_name(&boxed), "Alice");
    }

    #[test]
    fn test_demo() {
        demo();
    }

    /// The `#[implicit]` form of the same read.
    pub mod implicit {
        use cgp::prelude::*;

        #[cgp_component(Greeter)]
        pub trait CanGreet {
            fn greet(&self) -> String;
        }

        #[cgp_impl(new GreetHello)]
        impl Greeter {
            fn greet(&self, #[implicit] name: &str) -> String {
                format!("Hello, {name}!")
            }
        }

        #[derive(HasField)]
        pub struct Person {
            pub name: String,
        }

        delegate_components! {
            Person {
                GreeterComponent: GreetHello,
            }
        }

        #[test]
        fn the_implicit_argument_reads_the_same_field() {
            let person = Person {
                name: "Ada".to_owned(),
            };
            assert_eq!(person.greet(), "Hello, Ada!");
        }
    }
}
