use core::marker::PhantomData;

use cgp::prelude::*;

pub trait Database {
    type Row;
}

pub struct Pool<Db>(PhantomData<Db>);

// error[E0433]: cannot find type `Db` in this scope
#[cgp_fn]
#[impl_generics(Db: Database)]
pub fn fetch_row(&self, #[implicit] database: &Pool<Db>) -> Db::Row {
    let _ = database;
    todo!() // the page elides this body
}

fn main() {}
