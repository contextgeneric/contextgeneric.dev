use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self) -> String {
        "Hello!".to_owned()
    }
}

pub struct App;

// The entry alone, without the `IsProviderFor` impl `delegate_components!` emits beside it.
impl DelegateComponent<GreeterComponent> for App {
    type Delegate = GreetHello;
}

fn main() {
    let _ = App.greet();
}
