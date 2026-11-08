// From docs/projects/cgp-serde/guides/debugging-wiring.md: JSON error wiring is absent.
// Shows: the failed call reports the error components it requires.
use cgp::prelude::*;
use cgp_serde::components::ValueDeserializerComponent;
use cgp_serde::providers::UseSerde;
use cgp_serde_json::impls::CanDeserializeJsonString;

pub struct Ctx;

delegate_components! {
    Ctx {
        open ValueDeserializerComponent;
        @ValueDeserializerComponent.u64: UseSerde,
    }
}

fn main() {
    let _r: Result<u64, _> = Ctx.deserialize_json_string("1");
}
