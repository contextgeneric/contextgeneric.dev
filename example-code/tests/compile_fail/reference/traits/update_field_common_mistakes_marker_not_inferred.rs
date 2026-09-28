use cgp::prelude::*;

#[derive(BuildField)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
}

fn main() {
    // The target marker `M` is not inferred from the value.
    let (_old, _partial) =
        Person::builder().update_field(PhantomData::<Symbol!("first_name")>, "Alice".to_owned());
}
