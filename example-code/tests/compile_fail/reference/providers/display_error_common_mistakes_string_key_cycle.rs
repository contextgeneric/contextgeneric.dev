use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
use cgp::extra::error::DisplayError;
use cgp::prelude::*;

#[cgp_component(Loader)]
#[use_type(HasErrorType.Error)]
pub trait CanLoad {
    fn load(&self) -> Result<(), Error>;
}

#[cgp_impl(new LoadOrFail)]
#[uses(CanRaiseError<String>)]
#[use_type(HasErrorType.Error)]
impl Loader {
    fn load(&self) -> Result<(), Error> {
        Err(Self::raise_error("disk offline".to_owned()))
    }
}

pub struct App;

delegate_components! {
    App {
        open ErrorRaiserComponent;

        ErrorTypeProviderComponent: UseType<String>,
        LoaderComponent: LoadOrFail,

        @ErrorRaiserComponent.String: DisplayError,
    }
}

check_components! {
    App {
        LoaderComponent,
    }
}

fn main() {}
