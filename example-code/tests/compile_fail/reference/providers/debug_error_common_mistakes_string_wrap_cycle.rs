use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent, ErrorWrapperComponent};
use cgp::extra::error::{DebugError, RaiseFrom};
use cgp::prelude::*;

#[cgp_component(Loader)]
#[use_type(HasErrorType.Error)]
pub trait CanLoad {
    fn load(&self) -> Result<(), Error>;
}

#[cgp_impl(new LoadWithDetail)]
#[uses(CanRaiseError<String>, CanWrapError<String>)]
#[use_type(HasErrorType.Error)]
impl Loader {
    fn load(&self) -> Result<(), Error> {
        let error = Self::raise_error("disk offline".to_owned());
        Err(Self::wrap_error(error, "while loading config".to_owned()))
    }
}

pub struct App;

delegate_components! {
    App {
        open ErrorWrapperComponent;

        ErrorTypeProviderComponent: UseType<String>,
        ErrorRaiserComponent: RaiseFrom,
        LoaderComponent: LoadWithDetail,

        @ErrorWrapperComponent.String: DebugError,
    }
}

check_components! {
    App {
        LoaderComponent,
    }
}

fn main() {}
