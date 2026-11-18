use cgp::prelude::*;

#[cgp_type]
pub trait HasFooType {
    type Foo;
}

#[cgp_type]
pub trait HasBarType {
    type Bar;
}

// error[E0576]: cannot find associated type `Eror` in trait `HasErrorType`
#[cgp_fn]
#[use_type(HasErrorType.Eror)]
pub fn fail(&self) -> Result<(), Eror> {
    Ok(())
}

fn main() {}
