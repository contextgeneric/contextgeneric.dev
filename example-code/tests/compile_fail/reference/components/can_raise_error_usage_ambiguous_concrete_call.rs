use cgp::core::error::{ErrorRaiser, ErrorRaiserComponent, ErrorTypeProviderComponent};
use cgp::extra::error::RaiseFrom;
use cgp::prelude::*;

pub struct App;

delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<String>,
        ErrorRaiserComponent: RaiseFrom,
    }
}

fn main() {
    // error[E0034]: multiple applicable items in scope
    //
    // `App` implements the provider trait `ErrorRaiser` too, so with it imported the bare name is
    // ambiguous. `<App as CanRaiseError<String>>::raise_error(…)` names the consumer trait.
    let _ = App::raise_error("boom".to_owned());
}
