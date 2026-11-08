// From docs/reference/errors.md: when one mistake prints many errors.
// Shows: three checked components coalesce onto the same missing name field.
use cgp::prelude::*;

#[cgp_component(FooProvider)]
pub trait CanUseFoo {
    fn foo(&self);
}

#[cgp_component(BarProvider)]
pub trait CanUseBar {
    fn bar(&self);
}

#[cgp_component(BazProvider)]
pub trait CanUseBaz {
    fn baz(&self);
}

pub struct ReadName;

#[cgp_impl(ReadName)]
impl FooProvider {
    fn foo(&self, #[implicit] name: &str) {
        let _ = name;
    }
}

#[cgp_impl(ReadName)]
impl BarProvider {
    fn bar(&self, #[implicit] name: &str) {
        let _ = name;
    }
}

#[cgp_impl(ReadName)]
impl BazProvider {
    fn baz(&self, #[implicit] name: &str) {
        let _ = name;
    }
}

#[derive(HasField)]
pub struct App {
    pub age: u8,
}

delegate_components! {
    App {
        [FooProviderComponent, BarProviderComponent, BazProviderComponent]: ReadName,
    }
}

check_components! {
    App {
        BazProviderComponent,
        BarProviderComponent,
        FooProviderComponent,
    }
}
