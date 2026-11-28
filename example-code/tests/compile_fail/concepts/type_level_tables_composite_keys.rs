pub trait HasEntry<Key> {
    type Value;
}

pub struct MyTable;

pub struct FooKey;

pub struct Bar;
pub struct Baz;

pub struct BarKey;

impl HasEntry<(FooKey, BarKey)> for MyTable {
    type Value = Baz;
}

// error[E0119]: conflicting implementations of trait `HasEntry<(FooKey, BarKey)>` for type `MyTable`
impl<Key> HasEntry<(FooKey, Key)> for MyTable {
    type Value = Bar;
}

fn main() {}
