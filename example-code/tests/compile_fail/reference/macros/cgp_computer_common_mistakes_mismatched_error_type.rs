//! `docs/reference/macros/cgp_computer.md`, *Common Mistakes*: a `Result` function's error type must
//! be the context's error type, since the fallible bundles do not convert it.

use core::marker::PhantomData;

use cgp::core::error::ErrorTypeProviderComponent;
use cgp::extra::handler::TryComputer;
use cgp::prelude::*;

#[cgp_computer]
fn checked_add(a: u64, b: u64) -> Result<u64, String> {
    a.checked_add(b).ok_or_else(|| "overflow".to_owned())
}

pub struct App;

delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<u32>,
    }
}

fn main() {
    let _ = CheckedAdd::try_compute(&App, PhantomData::<()>, (1, 2));
}
