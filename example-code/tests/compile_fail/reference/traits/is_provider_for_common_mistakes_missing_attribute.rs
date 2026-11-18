use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

pub struct GreetHello;

// A provider trait impl written without `#[cgp_provider]` gets no `IsProviderFor` impl.
impl<Context> Greeter<Context> for GreetHello {
    fn greet(_context: &Context) -> String {
        "Hello!".to_owned()
    }
}

fn main() {}
