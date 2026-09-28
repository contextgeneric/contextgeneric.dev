// The raw pointer inside `Life` makes it neither `Send` nor `Sync`, and a provider struct that
// `#[cgp_impl(new ...)]` declares over a lifetime holds a `PhantomData<Life<'a>>`, so it is not
// `Send` either. This pins the `Life` page's *Definition*.

use cgp::prelude::*;

#[cgp_component(NameGetter)]
pub trait HasName<'a> {
    fn name(&self) -> &'a str;
}

#[cgp_impl(new GetName<'b>)]
impl<'a, 'b> NameGetter<'a> {
    fn name(&self) -> &'a str {
        "name"
    }
}

fn require_send<T: Send>() {}

fn main() {
    require_send::<Life<'static>>();
    require_send::<GetName<'static>>();
}
