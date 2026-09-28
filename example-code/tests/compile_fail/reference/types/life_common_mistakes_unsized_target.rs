// A component whose type parameter is `?Sized` passes `check_components!` at an unsized argument
// such as `str`, but a call through the wiring fails: the forwarding `IsProviderFor` impl that
// `delegate_components!` emits declares its `__Params__` parameter without `?Sized`, and the params
// tuple `(Life<'a>, str)` is unsized. This pins the `Life` page's *Common Mistakes* entry.

use cgp::prelude::*;

#[cgp_component(ReferenceGetter)]
pub trait HasReference<'a, T: 'a + ?Sized> {
    fn get_reference(&self) -> &'a T;
}

#[cgp_impl(new GetName)]
#[uses(HasField<Symbol!("name"), Value = &'a str>)]
impl<'a> ReferenceGetter<'a, str> {
    fn get_reference(&self) -> &'a str {
        self.get_field(PhantomData::<Symbol!("name")>)
    }
}

#[derive(HasField)]
pub struct Borrowed<'a> {
    pub name: &'a str,
}

delegate_components! {
    <'a> Borrowed<'a> {
        ReferenceGetterComponent: GetName,
    }
}

// The check passes.
check_components! {
    <'a> Borrowed<'a> {
        ReferenceGetterComponent: (Life<'a>, str),
    }
}

fn main() {
    let borrowed = Borrowed { name: "hello" };
    let name: &str = borrowed.get_reference();
    assert_eq!(name, "hello");
}
