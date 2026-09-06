//! Code from `docs/reference/attributes/impl_generics.md` — *`#[impl_generics(...)]`*.

/// ## Overview
///
/// The opening snippet, plus a context so the blanket impl has something to resolve against.
pub mod overview {
    use core::fmt::Display;

    use cgp::prelude::*;

    #[cgp_fn]
    #[impl_generics(Name: Display)]
    pub fn greet(&self, #[implicit] name: &Name) -> String {
        format!("Hello, {name}!")
    }

    #[derive(HasField)]
    pub struct Person {
        pub name: String,
    }

    #[test]
    fn greets_by_field() {
        let person = Person {
            name: "Alice".to_owned(),
        };
        assert_eq!(person.greet(), "Hello, Alice!");
    }
}

/// ## Usage
///
/// Two parameters, each pinned by its own implicit argument. The section's `#[cgp_impl]` remark
/// is checked here too: a provider declares the parameter in the block's own generic list.
pub mod usage {
    use core::fmt::Display;

    use cgp::prelude::*;

    #[cgp_fn]
    #[impl_generics(Name: Display, Count: Display)]
    pub fn describe(&self, #[implicit] name: &Name, #[implicit] count: &Count) -> String {
        format!("{count} × {name}")
    }

    #[derive(HasField)]
    pub struct Inventory {
        pub name: String,
        pub count: u32,
    }

    #[test]
    fn describes_two_fields() {
        let inventory = Inventory {
            name: "bolts".to_owned(),
            count: 12,
        };
        assert_eq!(inventory.describe(), "12 × bolts");
    }

    #[cgp_component(Greeter)]
    pub trait CanGreet {
        fn greet(&self) -> String;
    }

    #[cgp_impl(new GreetByField)]
    impl<Name: Display> Greeter {
        fn greet(&self, #[implicit] name: &Name) -> String {
            format!("Hello, {name}!")
        }
    }

    delegate_components! {
        Inventory {
            GreeterComponent: GreetByField,
        }
    }

    check_components! {
        Inventory {
            GreeterComponent,
        }
    }

    #[test]
    fn provider_declares_the_parameter_itself() {
        let inventory = Inventory {
            name: "bolts".to_owned(),
            count: 12,
        };
        assert_eq!(inventory.greet(), "Hello, bolts!");
    }
}

/// ## Examples
pub mod examples {
    use core::fmt::Display;

    use cgp::prelude::*;

    #[cgp_fn]
    #[impl_generics(Name: Display)]
    pub fn greet(&self, #[implicit] name: &Name) -> String {
        format!("Hello, {name}!")
    }

    #[derive(HasField)]
    pub struct Person {
        pub name: String,
    }

    #[derive(HasField)]
    pub struct Robot {
        pub name: u32,
    }

    pub fn greet_both(person: &Person, robot: &Robot) {
        println!("{}", person.greet()); // Hello, Alice!
        println!("{}", robot.greet()); // Hello, 42!
    }

    #[cgp_fn]
    #[uses(Greet)]
    pub fn announce(&self) -> String {
        format!("{} Welcome aboard.", self.greet())
    }

    #[test]
    fn both_contexts_greet() {
        let person = Person {
            name: "Alice".to_owned(),
        };
        let robot = Robot { name: 42 };
        assert_eq!(person.greet(), "Hello, Alice!");
        assert_eq!(robot.greet(), "Hello, 42!");
        assert_eq!(robot.announce(), "Hello, 42! Welcome aboard.");
        greet_both(&person, &robot);
    }
}

/// ## Under the hood
///
/// The expansion the page shows, written by hand, and checked to resolve the same way.
pub mod under_the_hood {
    use core::fmt::Display;
    use core::marker::PhantomData;

    use cgp::prelude::*;

    pub trait Greet {
        fn greet(&self) -> String;
    }

    impl<__Context__, Name: Display> Greet for __Context__
    where
        Self: HasField<Symbol!("name"), Value = Name>,
    {
        fn greet(&self) -> String {
            let name: &Name = self.get_field(PhantomData::<Symbol!("name")>);
            format!("Hello, {name}!")
        }
    }

    #[derive(HasField)]
    pub struct Person {
        pub name: String,
    }

    #[test]
    fn hand_written_expansion_resolves() {
        let person = Person {
            name: "Alice".to_owned(),
        };
        assert_eq!(person.greet(), "Hello, Alice!");
    }
}
