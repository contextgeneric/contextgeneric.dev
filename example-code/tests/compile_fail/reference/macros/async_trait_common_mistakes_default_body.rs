//! `docs/reference/macros/async_trait.md`, *Common Mistakes*: a default-bodied `async fn` keeps its
//! body while its signature is rewritten.

use cgp::prelude::*;

#[async_trait]
pub trait CanCount {
    async fn count(&self) -> u32 {
        1
    }
}

fn main() {}
