//! `docs/reference/macros/cgp_component.md`, *Common Mistakes*: two `#[use_type]` imports that
//! resolve to the same bare name are rejected.

use cgp::prelude::*;

#[cgp_component(Loader)]
#[use_type(HasErrorType.Error, HasErrorType.{Error})]
pub trait CanLoad {
    fn load(&self, path: &str) -> Result<String, Error>;
}

fn main() {}
