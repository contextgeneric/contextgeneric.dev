//! Code from `docs/reference/derives/derive_has_field.md` — *`#[derive(HasField)]`*.
//!
//! The page's *Under the hood* section shows the impls the derive generates. Those are not repeated
//! here: re-declaring them beside the derive that emits them would be a coherence conflict, so the
//! only honest check on that section is `cargo cgp expand`. What this file pins is every input shape
//! the page says is accepted, and that the generated access actually resolves.

/// ## What it's for
///
/// The page opens on the bound an implementation writes. The component and the context are shown
/// further down under *Examples*; they are declared here so the opening bound has something to hold.
pub mod what_its_for {
    use cgp::prelude::*;

    #[derive(HasField)]
    pub struct Person {
        pub name: String,
    }

    // The bound the page shows, exercised as a standalone function rather than left as prose.
    pub fn read_name<Context>(context: &Context) -> &String
    where
        Context: HasField<Symbol!("name"), Value = String>,
    {
        context.get_field(PhantomData::<Symbol!("name")>)
    }

    #[test]
    fn test_bound_resolves_for_a_deriving_struct() {
        let person = Person {
            name: "Alice".to_owned(),
        };

        assert_eq!(read_name(&person), "Alice");
    }
}

/// ## Using it
///
/// The four struct shapes the page lists, in the order it lists them: named fields, a raw-identifier
/// field, tuple fields, a unit struct, and a generic struct. The unit struct is the one whose whole
/// point is that it compiles and emits nothing.
pub mod using_it {
    use cgp::prelude::*;

    #[derive(HasField)]
    pub struct Person {
        pub name: String,
        pub age: u8,
    }

    #[derive(HasField)]
    pub struct Entry {
        pub r#type: String,
    }

    #[derive(HasField)]
    pub struct Rectangle(pub f64, pub f64);

    #[derive(HasField)]
    pub struct App;

    #[derive(HasField)]
    pub struct Wrapper<T> {
        pub value: T,
    }

    #[test]
    fn test_named_fields_are_keyed_by_symbol() {
        let person = Person {
            name: "Alice".to_owned(),
            age: 30,
        };

        assert_eq!(person.get_field(PhantomData::<Symbol!("name")>), "Alice");
        assert_eq!(*person.get_field(PhantomData::<Symbol!("age")>), 30);
    }

    #[test]
    fn test_a_raw_identifier_field_drops_the_prefix() {
        let entry = Entry {
            r#type: "text".to_owned(),
        };

        // `Symbol!("type")`, not `Symbol!("r#type")` — the page's rule.
        assert_eq!(entry.get_field(PhantomData::<Symbol!("type")>), "text");
    }

    #[test]
    fn test_tuple_fields_are_keyed_by_index() {
        let rectangle = Rectangle(3.0, 4.0);

        assert_eq!(*rectangle.get_field(PhantomData::<Index<0>>), 3.0);
        assert_eq!(*rectangle.get_field(PhantomData::<Index<1>>), 4.0);
    }

    #[test]
    fn test_generic_parameters_are_carried_through() {
        let wrapper = Wrapper { value: 42_u32 };

        assert_eq!(*wrapper.get_field(PhantomData::<Symbol!("value")>), 42);
    }
}

/// ## Examples
///
/// Both forms the page shows of one provider: the explicit `HasField` bound, and the `#[implicit]`
/// argument that generates the same bound. Two components are declared rather than one, since the
/// page shows the second form as a replacement for the first and one context cannot wire both.
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
            println!("Hello, {}!", self.get_field(PhantomData::<Symbol!("name")>));
        }
    }

    #[cgp_impl(new GreetHelloImplicit)]
    impl Greeter {
        fn greet(&self, #[implicit] name: &str) {
            println!("Hello, {name}!");
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

    /// The `#[implicit]` form, wired on its own context so the two providers do not collide.
    #[derive(HasField)]
    pub struct ImplicitPerson {
        pub name: String,
    }

    delegate_components! {
        ImplicitPerson {
            GreeterComponent: GreetHelloImplicit,
        }
    }

    check_components! {
        ImplicitPerson {
            GreeterComponent,
        }
    }

    #[test]
    fn test_both_forms_greet() {
        Person {
            name: "World".to_owned(),
        }
        .greet();

        ImplicitPerson {
            name: "World".to_owned(),
        }
        .greet();
    }

    /// The page's claim that field access passes through a smart pointer with no extra derive.
    #[test]
    fn test_access_through_a_box() {
        let boxed: Box<Person> = Box::new(Person {
            name: "Alice".to_owned(),
        });

        assert_eq!(boxed.get_field(PhantomData::<Symbol!("name")>), "Alice");
    }
}
