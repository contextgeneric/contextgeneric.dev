use cgp::core::field::traits::MapFields;
use cgp::prelude::*;
use cgp::core::field::impls::IsPresent;

// `MapFields` names its result `Mapped`; `Output` belongs to the other two operations.
pub type Mapped = <Product![u8, u16] as MapFields<IsPresent>>::Output;

fn main() {}
