// From docs/projects/cgp-serde/guides/debugging-wiring.md: a nested type has no entry.
// Shows: checking Payload names the missing Vec<u8> serialization entry.
use cgp::prelude::*;
use cgp_serde::components::ValueSerializerComponent;
use cgp_serde::providers::{SerializeRecordFields, UseSerde};

#[derive(CgpData)]
pub struct Payload {
    pub quantity: u64,
    pub data: Vec<u8>,
}

pub struct App;

delegate_components! {
    App {
        open ValueSerializerComponent;
        @ValueSerializerComponent.u64: UseSerde,
        @ValueSerializerComponent.Payload: SerializeRecordFields,
    }
}

check_components! {
    App {
        ValueSerializerComponent: [u64, Payload],
    }
}
