use cgp::prelude::*;

#[cgp_type]
pub trait HasFooType {
    type Foo;
}

#[cgp_type]
pub trait HasBarType {
    type Bar;
}

// error: Type equality constraints cannot be used in component trait definition
#[cgp_component(Loader)]
#[use_type(HasErrorType.{Error = anyhow::Error})]
pub trait CanLoad {
    fn load(&self, path: &str) -> Result<String, Error>;
}

fn main() {}
