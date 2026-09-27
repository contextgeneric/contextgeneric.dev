//! `docs/reference/macros/cgp_fn.md`, *Common Mistakes*: an `#[impl_generics]` parameter named in the
//! trait's own signature is out of scope there, bare (`E0425`) or qualified (`E0433`).

use cgp::prelude::*;

pub trait Database {
    type Row;
}

#[cgp_fn]
#[impl_generics(Db: Database)]
pub fn database_handle(&self, #[implicit] database: &Db) -> Db {
    todo!()
}

#[cgp_fn]
#[impl_generics(Db: Database)]
pub fn fetch_row(&self, #[implicit] database: &Db) -> Db::Row {
    todo!()
}

fn main() {}
