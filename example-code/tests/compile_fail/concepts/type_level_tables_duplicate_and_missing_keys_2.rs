use core::any::type_name;

pub trait HasEntry<Key> {
    type Value;
}

pub struct MyTable;

pub struct FooKey;

pub struct Bar;
pub struct Baz;

pub fn get_foo_value<Table: HasEntry<FooKey>>() -> &'static str {
    type_name::<Table::Value>()
}

pub struct OtherTable;

fn main() {
    // error[E0277]: the trait bound `OtherTable: HasEntry<FooKey>` is not satisfied
    get_foo_value::<OtherTable>();
}
