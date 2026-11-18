// From docs/projects/cgp-serde/guides/debugging-wiring.md: a key does not parse.
// Shows: the macro rejects a fixed-size array written as a list of keys.
use cgp::prelude::*;
use cgp_serde::components::ValueDeserializerComponent;
use cgp_serde::providers::UseSerde;

pub struct App;

delegate_components! {
    App {
        open ValueDeserializerComponent;
        @ValueDeserializerComponent.[u8; 32]: UseSerde,
    }
}
