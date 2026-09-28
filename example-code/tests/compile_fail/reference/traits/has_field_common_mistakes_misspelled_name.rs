use cgp::prelude::*;

pub fn read_first_name<Context>(context: &Context) -> &String
where
    Context: HasField<Symbol!("first_name"), Value = String>,
{
    context.get_field(PhantomData)
}

// The field is spelled `firstName`, a different type-level name.
#[allow(non_snake_case)]
#[derive(HasField)]
pub struct Person {
    pub firstName: String,
}

fn main() {
    let person = Person {
        firstName: "Ada".to_owned(),
    };
    let _ = read_first_name(&person);
}
