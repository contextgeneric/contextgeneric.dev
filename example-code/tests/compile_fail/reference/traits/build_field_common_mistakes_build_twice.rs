use cgp::prelude::*;

#[derive(BuildField)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
}

fn main() {
    // `first_name` is already present when the second `build_field` asks to set it.
    let _ = Person::builder()
        .build_field(PhantomData::<Symbol!("first_name")>, "Alice".to_owned())
        .build_field(PhantomData::<Symbol!("first_name")>, "Bob".to_owned());
}
