use cgp::prelude::*;

#[cgp_component {
    name: GreeterComponent,
    provider: Greeter,
}]
pub trait CanGreet {
    fn greet(&self);
}

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &String;
}

pub struct GreetHello;

impl<Context> Greeter<Context> for GreetHello
where
    Context: HasName,
{
    fn greet(context: &Context) {
        println!("Hello, {}!", context.name());
    }
}

#[derive(HasField)]
pub struct Person {
    pub name: String,
}

pub struct PersonComponents;

impl HasComponents for Person {
    type Components = PersonComponents;
}

delegate_components! {
    PersonComponents {
        GreeterComponent: GreetHello,
    }
}

fn main() {
    let person = Person {
        name: "Alice".into(),
    };

    person.greet();
}
