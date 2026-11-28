//! Code from `docs/concepts/type-level-tables.md` — *Look up a type in a type-level table*.
//!
//! Module names follow the page's headings. Each type-level step has a test, and each value-level
//! counterpart the page shows, reading a global `static` table, is a test beside it. The page uses no CGP, so neither does this file.
//! Rejected code lives in trybuild fixtures, except the cross-crate `E0210` and the `E0275` cycle,
//! which the page states in prose; the campaign's probe captured both messages.

/// ## A table and its entries
pub mod a_table_and_its_entries {
    use core::any::type_name;
    use std::collections::HashMap;
    use std::sync::LazyLock;

    pub trait HasEntry<Key> {
        type Value;
    }

    pub struct MyTable;

    pub struct FooKey;
    pub struct BarKey;

    pub struct Bar;
    pub struct Baz;

    impl HasEntry<FooKey> for MyTable {
        type Value = Bar;
    }

    impl HasEntry<BarKey> for MyTable {
        type Value = Baz;
    }

    pub fn get_foo_value<Table: HasEntry<FooKey>>() -> &'static str {
        type_name::<Table::Value>()
    }

    pub fn my_table_foo_value() -> &'static str {
        get_foo_value::<MyTable>()
    }

    /// The generic-key form the page describes in prose.
    pub fn get_value<Table: HasEntry<Key>, Key>() -> &'static str {
        type_name::<Table::Value>()
    }

    #[test]
    fn the_foo_entry_is_bar() {
        assert_eq!(my_table_foo_value(), type_name::<Bar>());
        assert_eq!(get_value::<MyTable, BarKey>(), type_name::<Baz>());
    }

    /// The page's value-level counterpart.
    pub fn get_foo_value_twin(table: &HashMap<&'static str, &'static str>) -> &'static str {
        table["FooKey"]
    }

    pub static MY_TABLE: LazyLock<HashMap<&'static str, &'static str>> =
        LazyLock::new(|| HashMap::from([("FooKey", "Bar"), ("BarKey", "Baz")]));

    pub fn my_table_foo_value_twin() -> &'static str {
        get_foo_value_twin(&MY_TABLE)
    }

    #[test]
    fn the_value_level_map_answers_the_same_key() {
        assert_eq!(my_table_foo_value_twin(), "Bar");
    }
}

/// ## Duplicate and missing keys
///
/// All three snippets are rejected:
///
/// Rejected snippets — trybuild fixtures
/// `tests/compile_fail/concepts/type_level_tables_duplicate_and_missing_keys_1.rs` (a second entry),
/// `_2.rs` (a missing key), and `_3.rs` (a generic key overlapping a specific one).
pub mod duplicate_and_missing_keys {
    use super::a_table_and_its_entries::MY_TABLE;

    /// The value-level map finds a missing key only at runtime.
    #[test]
    fn the_value_level_map_misses_at_runtime() {
        assert_eq!(MY_TABLE.get("OtherKey"), None);
    }
}

/// ## A lookup as a type
pub mod a_lookup_as_a_type {
    use core::any::type_name;
    use core::marker::PhantomData;
    use std::collections::HashMap;

    use super::a_table_and_its_entries::{Bar, BarKey, Baz, FooKey, HasEntry, MyTable, MY_TABLE};

    pub trait HasStaticString<Key> {
        fn static_string() -> &'static str;
    }

    pub struct Lookup<Table>(pub PhantomData<Table>);

    impl<Table, Key> HasStaticString<Key> for Lookup<Table>
    where
        Table: HasEntry<Key>,
    {
        fn static_string() -> &'static str {
            type_name::<Table::Value>()
        }
    }

    #[test]
    fn lookup_reads_the_table() {
        assert_eq!(
            <Lookup<MyTable> as HasStaticString<FooKey>>::static_string(),
            type_name::<Bar>(),
        );
        assert_eq!(
            <Lookup<MyTable> as HasStaticString<BarKey>>::static_string(),
            type_name::<Baz>(),
        );
    }

    pub fn lookup(table: &HashMap<&'static str, &'static str>, key: &str) -> &'static str {
        table[key]
    }

    #[test]
    fn the_value_level_lookup_takes_both_arguments() {
        assert_eq!(lookup(&MY_TABLE, "BarKey"), "Baz");
    }
}

/// ## Nested tables
pub mod nested_tables {
    use core::any::type_name;
    use core::marker::PhantomData;
    use std::collections::HashMap;
    use std::sync::LazyLock;

    use super::a_lookup_as_a_type::{HasStaticString, Lookup};
    use super::a_table_and_its_entries::{Bar, FooKey, HasEntry, MyTable, MY_TABLE};

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

    pub struct OuterTable;
    pub struct InnerKey;

    impl HasEntry<InnerKey> for OuterTable {
        type Value = Lookup<MyTable>;
    }

    #[test]
    fn forward_reads_both_tables() {
        assert_eq!(
            <Forward<OuterTable, InnerKey> as HasStaticString<FooKey>>::static_string(),
            type_name::<Bar>(),
        );
    }

    pub static OUTER_TABLE: LazyLock<HashMap<&'static str, &'static HashMap<&'static str, &'static str>>> =
        LazyLock::new(|| HashMap::from([("InnerKey", &*MY_TABLE)]));

    pub fn forward(
        outer: &HashMap<&'static str, &'static HashMap<&'static str, &'static str>>,
        key_b: &str,
        key_a: &str,
    ) -> &'static str {
        outer[key_b][key_a]
    }

    #[test]
    fn the_value_level_map_of_maps() {
        assert_eq!(forward(&OUTER_TABLE, "InnerKey", "FooKey"), "Bar");
    }
}

/// ## Composite keys
///
/// A generic position overlapping a specific pair is rejected:
///
/// Rejected snippet — trybuild fixture `tests/compile_fail/concepts/type_level_tables_composite_keys.rs`.
pub mod composite_keys {
    use core::any::type_name;
    use core::marker::PhantomData;
    use std::collections::HashMap;
    use std::sync::LazyLock;

    use super::a_lookup_as_a_type::HasStaticString;
    use super::a_table_and_its_entries::{BarKey, Baz, FooKey, HasEntry, MyTable};

    impl HasEntry<(FooKey, BarKey)> for MyTable {
        type Value = Baz;
    }

    pub struct Redirect<Table, Prefix>(pub PhantomData<(Table, Prefix)>);

    impl<Table, Prefix, Key> HasStaticString<Key> for Redirect<Table, Prefix>
    where
        Table: HasEntry<(Prefix, Key)>,
    {
        fn static_string() -> &'static str {
            type_name::<<Table as HasEntry<(Prefix, Key)>>::Value>()
        }
    }

    #[test]
    fn redirect_reads_the_pair() {
        assert_eq!(
            <Redirect<MyTable, FooKey> as HasStaticString<BarKey>>::static_string(),
            type_name::<Baz>(),
        );
    }

    pub static MY_PAIR_TABLE: LazyLock<HashMap<(&'static str, &'static str), &'static str>> =
        LazyLock::new(|| HashMap::from([(("FooKey", "BarKey"), "Baz")]));

    pub fn redirect(
        table: &HashMap<(&'static str, &'static str), &'static str>,
        prefix: &'static str,
        key: &'static str,
    ) -> &'static str {
        table[&(prefix, key)]
    }

    #[test]
    fn the_value_level_map_keyed_by_pairs() {
        assert_eq!(redirect(&MY_PAIR_TABLE, "FooKey", "BarKey"), "Baz");
    }
}

/// ## Entry ownership
///
/// The page's two legal impls. The page places them in a crate other than the one defining
/// `HasEntry` and `TheirTable`; this file is one crate, so `TheirTable` is declared here, where the
/// impls are legal too. The rejected `impl<Table> HasEntry<MyKey> for Table` is legal in one crate,
/// so it has no fixture: the page quotes its `E0210` from the campaign's two-crate probe.
pub mod entry_ownership {
    use super::a_table_and_its_entries::HasEntry;

    pub struct TheirTable;

    pub struct MyKey;
    pub struct MyValue;

    impl HasEntry<MyKey> for TheirTable {
        type Value = MyValue;
    }

    pub trait InTable<Table> {
        type Value;
    }

    impl<Table> InTable<Table> for MyKey {
        type Value = MyValue;
    }

    fn reads_entry<Table: HasEntry<MyKey, Value = MyValue>>() {}
    fn every_table_reads<Table>()
    where
        MyKey: InTable<Table, Value = MyValue>,
    {
    }

    #[test]
    fn both_entries_resolve() {
        reads_entry::<TheirTable>();
        every_table_reads::<TheirTable>();
        every_table_reads::<()>();
    }
}
