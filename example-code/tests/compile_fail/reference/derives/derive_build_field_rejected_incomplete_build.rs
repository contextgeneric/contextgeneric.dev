use cgp::prelude::*;

#[derive(BuildField)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
}

fn main() {
    // `finalize_build` is called without setting `last_name`.
    let person: Person = Person::builder()
        .build_field(PhantomData::<Symbol!("first_name")>, "Alice".to_owned())
        .finalize_build();
}
