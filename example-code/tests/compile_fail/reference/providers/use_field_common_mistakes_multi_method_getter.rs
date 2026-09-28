use cgp::prelude::*;

#[cgp_getter]
pub trait HasFooBar {
    fn foo(&self) -> &str;
    fn bar(&self) -> &u8;
}

#[derive(HasField)]
pub struct App {
    pub foo: String,
    pub bar: u8,
}

delegate_components! {
    App {
        FooBarGetterComponent: UseField<Symbol!("foo")>,
    }
}

check_components! {
    App {
        FooBarGetterComponent,
    }
}

fn main() {}
