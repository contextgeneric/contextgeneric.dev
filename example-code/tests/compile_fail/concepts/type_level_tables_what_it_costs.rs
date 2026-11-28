use core::marker::PhantomData;

pub trait HasEntry<Key> {
    type Value;
}

pub trait HasStaticString<Key> {
    fn static_string() -> &'static str;
}

pub struct Forward<Table, KeyB>(pub PhantomData<(Table, KeyB)>);

impl<Table, KeyA, KeyB> HasStaticString<KeyA> for Forward<Table, KeyB>
where
    Table: HasEntry<KeyB>,
    Table::Value: HasStaticString<KeyA>,
{
    fn static_string() -> &'static str {
        <Table::Value as HasStaticString<KeyA>>::static_string()
    }
}

pub struct FooKey;

pub struct Loop;

impl HasEntry<FooKey> for Loop {
    type Value = Forward<Loop, FooKey>;
}

fn main() {
    // error[E0275]: overflow evaluating the requirement `Forward<Loop, FooKey>: HasStaticString<FooKey>`
    <Forward<Loop, FooKey> as HasStaticString<FooKey>>::static_string();
}
