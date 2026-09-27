use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self, #[implicit] name: &str) {
        println!("Hello, {name}!");
    }
}

#[derive(HasField)]
pub struct Person {
    pub first_name: String, // GreetHello needs `name`
}

delegate_components! {
    Person {
        GreeterComponent: GreetHello,
    }
}

// error[E0277]: `Person` does not implement `HasField` for the `name` tag
check_components! {
    Person {
        GreeterComponent,
    }
}

fn main() {}
