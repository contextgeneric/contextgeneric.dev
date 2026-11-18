use cgp::prelude::*;

cgp_namespace! { new FirstNamespace {} }

cgp_namespace! { new SecondNamespace {} }

pub struct App;

// Each `namespace` line emits a blanket `DelegateComponent` impl covering every key, so the two
// forwarding impls overlap.
delegate_components! {
    App {
        namespace FirstNamespace;
        namespace SecondNamespace;
    }
}

fn main() {}
