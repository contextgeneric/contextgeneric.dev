// From docs/projects/cgp-serde/guides/debugging-wiring.md: an iterator yields references.
// Shows: checking Vec<u64> names the missing &u64 serialization entry.
use cgp::prelude::*;
use cgp_serde::components::ValueSerializerComponent;
use cgp_serde::providers::{SerializeIterator, UseSerde};

pub struct Ctx;

delegate_components! {
    Ctx {
        open ValueSerializerComponent;
        @ValueSerializerComponent.u64: UseSerde,
        @ValueSerializerComponent.Vec<u64>: SerializeIterator,
    }
}

check_components! {
    Ctx {
        ValueSerializerComponent: Vec<u64>,
    }
}
