use cgp::prelude::*;

#[cgp_component(Greeter)]
#[prefix(@app in DefaultNamespace)]
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

// App joins the namespace, so the lookup follows the redirect to `@app.GreeterComponent`, but
// nothing binds a provider at that path.
delegate_components! {
    App {
        namespace DefaultNamespace;
    }
}

// error[E0277]: the trait bound `PathCons<...>: DefaultNamespace<App>` is not satisfied
check_components! {
    App {
        GreeterComponent,
    }
}

fn main() {}
