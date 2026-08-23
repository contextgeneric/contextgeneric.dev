use cgp::prelude::*;

#[derive(BuildField)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
}

fn main() {
    let partial = Person::builder()
        .build_field(PhantomData::<Symbol!("first_name")>, "Alice".to_owned());

    // Reading a field that was never set.
    let _ = partial.get_field(PhantomData::<Symbol!("last_name")>);
}
