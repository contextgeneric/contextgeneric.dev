use cgp::prelude::*;

#[cgp_component(ShowImpl)]
#[prefix(@show in AppNamespace)]
pub trait CanShow {
    fn show(&self) -> String;
}

cgp_namespace! {
    new AppNamespace {}
}

#[derive(Debug)]
pub struct MyApp;

// Nothing binds a provider at `@show.ShowImplComponent`.
delegate_components! {
    MyApp {
        namespace AppNamespace;
    }
}

// error[E0277]: the path `@show.ShowImplComponent` is not something the namespace resolves
check_components! {
    MyApp {
        ShowImplComponent,
    }
}

fn main() {}
