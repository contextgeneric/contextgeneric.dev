use cgp::prelude::*; // Import all CGP constructs

// Derive CGP provider traits and blanket implementations
#[cgp_component {
    name: GreeterComponent, // Name of the CGP component
    provider: Greeter, // Name of the provider trait
}]
pub trait CanGreet // Name of the consumer trait
{
    fn greet(&self);
}

// A getter trait representing a dependency for `name` value
#[cgp_auto_getter] // Derive blanket implementation
pub trait HasName {
    fn name(&self) -> &String;
}

// A provider that implements `Greeter`
pub struct GreetHello;

// Implement `Greeter` that is generic over `Context`
impl<Context> Greeter<Context> for GreetHello
where
    Context: HasName, // Inject the `name` dependency from `Context`
{
    fn greet(context: &Context) {
        // `self` is replaced by `context` inside providers
        println!("Hello, {}!", context.name());
    }
}

// A concrete context that uses CGP components
#[derive(HasField)] // Deriving `HasField` automatically implements `HasName`
pub struct Person {
    pub name: String,
}

// The CGP components mapping for `Person`
pub struct PersonComponents;

// Set CGP components used by `Person` to be `PersonComponents`
impl HasComponents for Person {
    type Components = PersonComponents;
}

// Compile-time wiring of CGP components
delegate_components! {
    PersonComponents {
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
