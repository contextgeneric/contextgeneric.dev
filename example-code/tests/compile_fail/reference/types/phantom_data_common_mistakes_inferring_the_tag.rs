// `get_field` takes its tag as a `PhantomData<Tag>` argument. On a context with more than one field,
// a bare `PhantomData` leaves the tag to inference, and nothing fixes it. This is the `PhantomData`
// page's *Common Mistakes* entry on the turbofish.

use cgp::prelude::*;

#[derive(HasField)]
pub struct Person {
    pub name: String,
    pub age: u8,
}

pub fn print_name(person: &Person) {
    let name = person.get_field(PhantomData);
    println!("{name}");
}

fn main() {}
