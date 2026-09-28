use cgp::prelude::*;

// Two chained reads tie the inner borrow to a lifetime `Inner` is not known to outlive.
pub fn inner_name<Context, Inner>(context: &Context) -> &String
where
    Context: HasField<Symbol!("inner"), Value = Inner>,
    Inner: HasField<Symbol!("name"), Value = String>,
{
    context
        .get_field(PhantomData::<Symbol!("inner")>)
        .get_field(PhantomData::<Symbol!("name")>)
}

fn main() {}
