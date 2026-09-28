use cgp::core::field::traits::TakeField;
use cgp::prelude::*;

#[derive(BuildField)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
}

fn main() {
    // An empty builder has no `first_name` to take.
    let _ = Person::builder().take_field(PhantomData::<Symbol!("first_name")>);
}
