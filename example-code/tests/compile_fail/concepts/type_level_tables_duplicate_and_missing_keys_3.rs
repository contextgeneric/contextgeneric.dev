pub trait HasEntry<Key> {
    type Value;
}

pub struct MyTable;

pub struct FooKey;

pub struct Bar;
pub struct Baz;

impl<T> HasEntry<Vec<T>> for MyTable {
    type Value = Bar;
}

// error[E0119]: conflicting implementations of trait `HasEntry<Vec<u8>>` for type `MyTable`
impl HasEntry<Vec<u8>> for MyTable {
    type Value = Baz;
}

fn main() {}
