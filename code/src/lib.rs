use cgp::prelude::*;

#[cgp_getter {
    provider: NameGetter,
}]
pub trait HasName {
    fn name(&self) -> &String;
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
        NameGetterComponent: UseFields,
    }
}

fn main() {
    let person = Person {
        name: "Alice".into(),
    };

    println!("Hello, {}", person.name());
}
