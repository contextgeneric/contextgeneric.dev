use cgp::prelude::*;

#[cgp_getter(NameGetter)]
pub trait HasName {
    fn name(&self) -> &String;
}

pub struct Profile {
    pub display_name: String,
}

#[derive(HasField)]
pub struct Account {
    pub profile: Profile,
}

pub struct ReadDisplayName;

impl<Context, Tag> FieldGetter<Context, Tag> for ReadDisplayName
where
    Context: HasField<Symbol!("profile"), Value = Profile>,
{
    type Value = String;

    fn get_field(context: &Context, _tag: PhantomData<Tag>) -> &String {
        &context.get_field(PhantomData).display_name
    }
}

// A `FieldGetter` provider does not implement the getter's provider trait itself.
delegate_components! {
    Account {
        NameGetterComponent: ReadDisplayName,
    }
}

check_components! {
    Account {
        NameGetterComponent,
    }
}

fn main() {}
