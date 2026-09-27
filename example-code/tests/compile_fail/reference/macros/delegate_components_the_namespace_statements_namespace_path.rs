use cgp::prelude::*;

mod some_mod {
    pub use cgp::prelude::DefaultNamespace as SomeNamespace;
}

pub struct App;

// error: expected `;`, at the `::`
delegate_components! {
    App {
        namespace some_mod::SomeNamespace;
    }
}

fn main() {}
