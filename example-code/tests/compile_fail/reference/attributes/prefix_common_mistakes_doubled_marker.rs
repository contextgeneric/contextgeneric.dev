use core::fmt::Display;

use cgp::prelude::*;

// The prefix repeats the marker the macro appends, so the route is
// `@app.ShowImplComponent.ShowImplComponent.String`, which the entry below does not match.
#[cgp_component(ShowImpl)]
#[prefix(@app.ShowImplComponent in DefaultNamespace)]
pub trait CanShow<T> {
    fn show(&self, value: &T) -> String;
}

#[cgp_impl(new ShowWithDisplay)]
impl<T: Display> ShowImpl<T> {
    fn show(&self, value: &T) -> String {
        format!("{value}")
    }
}

pub struct App;

delegate_components! {
    App {
        namespace DefaultNamespace;

        @app.ShowImplComponent.String: ShowWithDisplay,
    }
}

// error[E0277]: the trait bound `PathCons<...>: DefaultNamespace<App>` is not satisfied
check_components! {
    App {
        ShowImplComponent: String,
    }
}

fn main() {}
