use cgp::core::error::{ErrorTypeProviderComponent, ErrorWrapperComponent};
use cgp::extra::error::DisplayError;
use cgp::prelude::*;

pub struct App;

// `DisplayError` forwards to the context's own `CanWrapError<String>`, which is this same entry.
delegate_components! {
    App {
        open ErrorWrapperComponent;

        ErrorTypeProviderComponent: UseType<String>,
        @ErrorWrapperComponent.String: DisplayError,
    }
}

// error[E0275]: overflow evaluating the requirement `App: IsProviderFor<ErrorWrapperComponent, App, String>`
check_components! {
    App {
        ErrorWrapperComponent: String,
    }
}

fn main() {}
