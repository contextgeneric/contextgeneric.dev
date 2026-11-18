use cgp::core::error::{ErrorTypeProviderComponent, ErrorWrapper, ErrorWrapperComponent};
use cgp::extra::error::DiscardDetail;
use cgp::prelude::*;

pub struct App;

delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<String>,
        ErrorWrapperComponent: DiscardDetail,
    }
}

fn main() {
    // error[E0034]: multiple applicable items in scope
    let _ = App::wrap_error("failed".to_owned(), "detail".to_owned());
}
