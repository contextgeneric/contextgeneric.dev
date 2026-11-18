//! Code from `docs/concepts/send-bounds.md` — *Recovering `Send` bounds*.

use core::future::Future;

use cgp::prelude::*;

#[cgp_component(ApiHandler)]
#[async_trait]
pub trait CanHandleApi<Api> {
    type Response;

    async fn handle_api(&self, _api: PhantomData<Api>) -> Self::Response;
}

pub struct QueryBalance;

#[cgp_impl(new HandleQueryBalance)]
impl ApiHandler<QueryBalance> {
    type Response = u64;

    async fn handle_api(&self, _api: PhantomData<QueryBalance>) -> u64 {
        100
    }
}

pub struct MockApp;

delegate_components! {
    MockApp {
        open ApiHandlerComponent;

        @ApiHandlerComponent.QueryBalance: HandleQueryBalance,
    }
}

mod check_mock_app {
    use super::*;
    check_components! {
        MockApp {
            ApiHandlerComponent: QueryBalance,
        }
    }
}

/// ## The bound a spawning caller needs, and cannot write
///
/// A work-stealing executor requires the future it drives to be `Send`. A caller generic over the
/// context cannot say so, because the future's auto-traits are hidden behind the trait boundary —
/// the notation for it, Return Type Notation, is not stable.
///
/// Rejected snippet — trybuild fixture `tests/compile_fail/concepts/send_bounds_the_bound_a_caller_cannot_write.rs`.
pub mod the_bound_a_caller_cannot_write {}

/// ## The trait that states the bound directly
///
/// An ordinary trait — not a component — whose method spells `+ Send` on its return type, which
/// sidesteps the missing notation.
pub trait CanHandleApiSend<Api>: CanHandleApi<Api> + Send + Sync
where
    Self::Response: Send,
{
    fn handle_api_send(&self, api: PhantomData<Api>)
        -> impl Future<Output = Self::Response> + Send;
}

/// The implementation has to be concrete. At a fixed context and a fixed API the call resolves to a
/// concrete provider producing a concrete future, and the compiler works out for itself that it is
/// `Send`.
impl CanHandleApiSend<QueryBalance> for MockApp {
    async fn handle_api_send(&self, api: PhantomData<QueryBalance>) -> u64 {
        self.handle_api(api).await
    }
}

/// ## What the recovered bound buys
///
/// A caller can now demand the `Send` future by name.
pub fn requires_a_send_future<App, Api>(app: &App)
where
    App: CanHandleApiSend<Api>,
    App::Response: Send,
{
    let future = app.handle_api_send(PhantomData::<Api>);
    fn assert_send<T: Send>(_: &T) {}
    assert_send(&future);
}

#[test]
fn the_concrete_impl_satisfies_the_bound() {
    requires_a_send_future::<MockApp, QueryBalance>(&MockApp);
}
