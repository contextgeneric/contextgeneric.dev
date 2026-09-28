use cgp::core::field::impls::WithFieldRef;
use cgp::prelude::*;

#[cgp_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[derive(HasField)]
pub struct Person {
    pub name: String,
}

delegate_components! {
    Person {
        NameGetterComponent: WithFieldRef<Symbol!("name"), str>,
    }
}

check_components! {
    Person {
        NameGetterComponent,
    }
}

fn main() {}
