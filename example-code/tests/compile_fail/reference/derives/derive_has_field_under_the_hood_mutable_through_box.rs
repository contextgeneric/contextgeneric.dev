use cgp::prelude::*;

#[derive(HasField)]
pub struct Borrowed<'a> {
    pub name: &'a str,
}

// error[E0521]: borrowed data escapes outside of function
//
// The mutable `Deref` blanket impl requires a `'static` target, so `Borrowed<'a>` behind a `Box`
// can be read but not written through it.
pub fn rename<'a>(boxed: &mut Box<Borrowed<'a>>) {
    let _ = boxed.get_field_mut(PhantomData::<Symbol!("name")>);
}

fn main() {}
