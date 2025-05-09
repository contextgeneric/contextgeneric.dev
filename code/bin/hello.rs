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
    fn name(&self) -> &String;
}

// Implement `Greeter` that is generic over `Context`
#[cgp_new_provider]
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
#[cgp_context]
#[derive(HasField)] // Deriving `HasField` automatically implements `HasName`
pub struct Person {
    pub name: String,
}

// Compile-time wiring and checking of CGP components
delegate_and_check_components! {
    CanUsePerson for Person;
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
