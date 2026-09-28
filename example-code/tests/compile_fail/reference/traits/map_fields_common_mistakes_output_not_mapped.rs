use cgp::core::field::traits::MapFields;
use cgp::prelude::*;

// `MapFields` names its result `Mapped`; `Output` belongs to the other two operations.
pub type Mapped = <Product![u8, u16] as MapFields<IsPresent>>::Output;

fn main() {}
