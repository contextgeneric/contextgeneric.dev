use cgp::prelude::*;

#[derive(BuildField)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
}

fn main() {
    // `Person` declares no `age` field.
    let _ = Person::builder().build_field(PhantomData::<Symbol!("age")>, 42_u8);
}
