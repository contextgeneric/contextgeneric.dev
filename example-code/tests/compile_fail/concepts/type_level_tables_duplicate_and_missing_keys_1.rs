pub trait HasEntry<Key> {
    type Value;
}

pub struct MyTable;

pub struct FooKey;

pub struct Bar;
pub struct Baz;

impl HasEntry<FooKey> for MyTable {
    type Value = Bar;
}

// error[E0119]: conflicting implementations of trait `HasEntry<FooKey>` for type `MyTable`
impl HasEntry<FooKey> for MyTable {
    type Value = Baz;
}

fn main() {}
