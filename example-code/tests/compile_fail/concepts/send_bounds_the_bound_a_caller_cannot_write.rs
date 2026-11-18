use core::marker::PhantomData;

use cgp::prelude::*;

#[cgp_component(ApiHandler)]
#[async_trait]
pub trait CanHandleApi<Api> {
    type Response;

    async fn handle_api(&self, _api: PhantomData<Api>) -> Self::Response;
}

// Not available on stable Rust: the return-type-notation bound `handle_api(..): Send`.
fn spawn_handler<App, Api>(app: App)
where
    App: CanHandleApi<Api, handle_api(..): Send> + Send + 'static,
{
}

fn main() {}
