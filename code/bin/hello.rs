use cgp::prelude::*; // Import all CGP constructs

// Derive CGP provider traits and blanket implementations
#[cgp_component(Greeter)]
pub trait CanGreet // Name of the consumer trait
{
    fn greet(&self);
}

// A getter trait representing a dependency for `name` value
#[cgp_auto_getter] // Derive blanket implementation
pub trait HasName {
    fn name(&self) -> &str;
}

// Implement `Greeter` that is generic over `Context`
#[cgp_impl(new GreetHello)]
impl<Context> Greeter for Context
where
    Context: HasName, // Inject the `name` dependency from `Context`
{
    fn greet(&self) {
        println!("Hello, {}!", self.name());
    }
}

// A concrete context that uses CGP components
#[derive(HasField)] // Deriving `HasField` automatically implements `HasName`
pub struct Person {
    pub name: String,
}

// Compile-time wiring of CGP components
delegate_components! {
    Person {
        GreeterComponent: GreetHello, // Use `GreetHello` to provide `Greeter`
    }
}

fn main() {
    let person = Person {
        name: "Alice".into(),
    };

    // `CanGreet` is automatically implemented for `Person`
    person.greet();
}
