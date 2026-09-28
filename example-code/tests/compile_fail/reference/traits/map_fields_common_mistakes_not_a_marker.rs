use cgp::core::field::traits::MapFields;
use cgp::prelude::*;

// A plain struct, with no `MapType` impl.
pub struct Wrapped;

pub fn take(_fields: <Product![u8, u16] as MapFields<Wrapped>>::Mapped) {}

fn main() {}
