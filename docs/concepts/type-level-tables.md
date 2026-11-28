---
title: 'Look up a type in a type-level table'
sidebar_label: 'Type-level tables'
sidebar_position: 3
description: 'How a trait with an associated type works as a compile-time map, shown in plain Rust beside the value-level code it mirrors.'
---

# Type-level tables

[Context-Generic Programming](/docs/) (CGP) extends Rust with pluggable trait implementations, and a
context's wiring records which implementation it selects for each component. That record is a
table, but not one the running program holds. It is a set of trait implementations that the
compiler looks up while it checks types.

This page shows how a trait can hold such a table, in plain Rust and without any CGP. Each
type-level step returns a `&'static str`, so you can compare it with the ordinary value-level code it
mirrors, written with a `HashMap`. The tables here are plain types, not contexts. A short section
near the end maps the pieces onto the traits CGP uses, and the pages it links show those traits at
work.

## A table and its entries

A trait whose only item is an associated type already works as a map. Each implementation is one
entry: the implementing type is the table, the trait's type parameter is the key, and the associated
type is the value. This table has two entries, which map `FooKey` to `Bar` and `BarKey` to `Baz`:

```rust
use core::any::type_name;

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

#[test]
fn the_foo_entry_is_bar() {
    assert_eq!(my_table_foo_value(), type_name::<Bar>());
}
```

Reading an entry takes two steps. The bound `Table: HasEntry<FooKey>` asks whether the table has an
entry for `FooKey`, and `Table::Value` names the value stored there. That short form stands for
`<Table as HasEntry<FooKey>>::Value`, and Rust accepts it because only one bound in scope supplies a
`Value`. The function then turns the type into a string with
[`type_name`](https://doc.rust-lang.org/std/any/fn.type_name.html), which returns the type's full path
rather than the bare name `Bar`. That is why the test compares the result with `type_name::<Bar>()`.

The same lookup at the value level is an ordinary map from strings to strings:

```rust
use std::collections::HashMap;

fn get_foo_value(table: &HashMap<&'static str, &'static str>) -> &'static str {
    table["FooKey"]
}

let my_table = HashMap::from([("FooKey", "Bar"), ("BarKey", "Baz")]);
assert_eq!(get_foo_value(&my_table), "Bar");
```

The value-level function receives its table as an argument, while the type-level one receives it as
the type parameter `Table`. The key can be a parameter too: `get_value<Table: HasEntry<Key>, Key>()`
reads whichever entry its caller names, as a value-level function would take the key as a second
argument.

## Duplicate and missing keys

[Coherence](/docs/reference/glossary#coherence), Rust's rule that a trait has at most one
implementation for a given type and set of parameters, is what makes this trait a map rather than a
list of pairs. A second entry for the same key would be a second implementation of
`HasEntry<FooKey>` for `MyTable`, and the compiler rejects it:

```rust
impl HasEntry<FooKey> for MyTable {
    type Value = Bar;
}

// error[E0119]: conflicting implementations of trait `HasEntry<FooKey>` for type `MyTable`
impl HasEntry<FooKey> for MyTable {
    type Value = Baz;
}
```

A missing key fails at compile time as well. Asking a table that has no `FooKey` entry for one
does not compile:

```rust
pub struct OtherTable;

// error[E0277]: the trait bound `OtherTable: HasEntry<FooKey>` is not satisfied
get_foo_value::<OtherTable>();
```

The `HashMap` version finds the same mistake only when the program runs, where `table["FooKey"]`
panics and `table.get("FooKey")` returns `None`.

Keys can be generic, and a generic key covers every type it matches. An entry for every `Vec<T>`
therefore overlaps an entry for `Vec<u8>`, and the compiler rejects the pair rather than choosing the
more specific one:

```rust
impl<T> HasEntry<Vec<T>> for MyTable {
    type Value = Bar;
}

// error[E0119]: conflicting implementations of trait `HasEntry<Vec<u8>>` for type `MyTable`
impl HasEntry<Vec<u8>> for MyTable {
    type Value = Baz;
}
```

The compiler resolves every lookup while it checks types, so the compiled program contains no table
and never consults one.

## A lookup as a type

A type can also answer a question by reading a table it is given. The trait `HasStaticString<Key>`
works as a function from a key to a string, and `Lookup<Table>` implements it by reading `Table`:

```rust
use core::marker::PhantomData;

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
}
```

The `where` clause is a precondition that the compiler checks at each use: `Lookup<MyTable>` answers
`FooKey` because `MyTable` has an entry for it, and it would not compile for a key the table lacks.
[`PhantomData`](https://doc.rust-lang.org/std/marker/struct.PhantomData.html) lets the struct name
`Table` without storing a value of it, since the table exists only as a type.

At the value level, the table and the key are both arguments:

```rust
fn lookup(table: &HashMap<&'static str, &'static str>, key: &str) -> &'static str {
    table[key]
}

assert_eq!(lookup(&my_table, "BarKey"), "Baz");
```

## Nested tables

An entry's value can itself answer a question, which gives a lookup in two steps. `Forward<Table,
KeyB>` reads `Table`'s entry for `KeyB`, then asks the type it finds there about `KeyA`:

```rust
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
```

The outer table's entry for `InnerKey` is `Lookup<MyTable>`, so `Forward<OuterTable, InnerKey>`
answers `FooKey` by reading `MyTable`. Each step is its own bound in the `where` clause, and the
compiler proves both.

At the value level, the same shape is a map of maps:

```rust
fn forward(
    outer: &HashMap<&'static str, HashMap<&'static str, &'static str>>,
    key_b: &str,
    key_a: &str,
) -> &'static str {
    outer[key_b][key_a]
}

let outer = HashMap::from([("InnerKey", my_table.clone())]);
assert_eq!(forward(&outer, "InnerKey", "FooKey"), "Bar");
```

## Composite keys

A key can also be made of several types. A tuple of two keys is one key, so a single entry can
answer a pair:

```rust
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
```

`Redirect<MyTable, FooKey>` fixes the first half of the key and takes the second half as its
question, so asking it about `BarKey` reads the entry for `(FooKey, BarKey)`. The listing writes the
projection in full to show which key the bound names, although `Table::Value` would also compile here,
since one bound supplies it.

At the value level, the key is a tuple of strings:

```rust
fn redirect(
    table: &HashMap<(&'static str, &'static str), &'static str>,
    prefix: &'static str,
    key: &'static str,
) -> &'static str {
    table[&(prefix, key)]
}

let pairs = HashMap::from([(("FooKey", "BarKey"), "Baz")]);
assert_eq!(redirect(&pairs, "FooKey", "BarKey"), "Baz");
```

A generic position in a composite key matches every type there, as a generic key did. An entry for
every pair that starts with `FooKey` therefore overlaps the entry for `(FooKey, BarKey)`:

```rust
// error[E0119]: conflicting implementations of trait `HasEntry<(FooKey, BarKey)>` for type `MyTable`
impl<Key> HasEntry<(FooKey, Key)> for MyTable {
    type Value = Bar;
}
```

## Entry ownership

Rust's [orphan rules](https://doc.rust-lang.org/reference/items/implementations.html#orphan-rules)
decide which crate may write an entry, because an entry is an implementation. A crate that owns a key
type can add an entry for it to a table it does not own:

```rust
// In a crate that depends on the crate defining `HasEntry` and `TheirTable`.
pub struct MyKey;
pub struct MyValue;

impl HasEntry<MyKey> for TheirTable {
    type Value = MyValue;
}
```

That crate cannot, however, add one entry for every table at once. In a crate other than the one
that defines `HasEntry`, `impl<Table> HasEntry<MyKey> for Table` fails with `E0210`: "type parameter
`Table` must be covered by another type when it appears before the first local type (`MyKey`)".

Turning the trait around removes that limit. When the key implements the trait and the table is its
parameter, the key's crate can write one entry that every table reads:

```rust
pub trait InTable<Table> {
    type Value;
}

impl<Table> InTable<Table> for MyKey {
    type Value = MyValue;
}
```

`MyKey` is local and comes first in this implementation, so the orphan rules accept it for every
`Table`. The table is written from the key's side.

## The tables in CGP

CGP's wiring is built from these pieces under other names. Each item below links the page that shows
it at work:

- **`HasEntry<Key>`** is [`DelegateComponent<Key>`](/docs/reference/traits/wiring/delegate_component),
  with the context as the table and a component as the key.
  [`delegate_components!`](/docs/reference/macros/delegate_components) writes one implementation per
  entry.
- **`Forward<Table, Key>`** is the provider blanket implementation that a component generates: it
  reads the context's entry for the component and asks the provider it finds there.
  [Consumer and provider traits](./consumer-and-provider-traits.md#how-a-call-finds-its-provider)
  follows a call through it.
- **`Lookup<Table>`** is the legacy [`UseDelegate`](/docs/reference/providers/use_delegate), so the
  outer table whose entry is a `Lookup` has the shape of a per-type dispatch table. An
  [aggregate provider](./aggregate-providers.md) is the case where the entry is another table, asked
  about the same key.
- **`Redirect<Table, Prefix>`** is [`RedirectLookup`](/docs/reference/providers/redirect_lookup),
  behind the `open` statement and [namespaces](./namespaces.md#presets-and-per-type-dispatch). Its
  keys are type-level paths, whose last position is generic, so a shorter path covers every longer
  one beneath it.
- **`InTable<Table>`** is the shape of a [namespace](./namespaces.md#registering-from-the-components-side)
  entry, which a component's crate writes from the key's side.

## What it costs

A table cannot express a preference. Every overlap is an error, whether between two entries or
between a generic entry and a specific one, so a table cannot hold a default beside the cases that
override it.

Each lookup is a trait obligation, which the compiler proves rather than evaluates. A lookup that
needs its own answer overflows the trait solver. A table whose entry is a `Forward` back to the same
table and key, for example, fails with `E0275`: "overflow evaluating the requirement
`Forward<Loop, FooKey>: HasStaticString<FooKey>`". CGP meets the same limit with recursive types, and
[Rust's own proposals](/docs/comparisons/rust-language-proposals#the-context-as-a-dictionary)
describes the workaround.

The table is also fixed when the program compiles. Unlike its `HashMap` counterpart, it cannot be
listed, iterated, or changed while the program runs, and each lookup adds work for the compiler.

## Where to go next

These pages show the same tables in CGP's wiring:

- [Consumer and provider traits](./consumer-and-provider-traits.md): How a call reaches its provider
  through a context's table.
- [Aggregate providers](./aggregate-providers.md): A table used as a provider and shared between
  contexts.
- [Namespaces](./namespaces.md): Tables written from the key's side, and path keys.
- [`DelegateComponent`](/docs/reference/traits/wiring/delegate_component) and
  [`delegate_components!`](/docs/reference/macros/delegate_components): The table trait and the macro
  that writes it.
- [`RedirectLookup`](/docs/reference/providers/redirect_lookup),
  [`UseDelegate`](/docs/reference/providers/use_delegate), [`Path!`](/docs/reference/macros/path), and
  [`PathCons`](/docs/reference/types/path_cons): Path keys and nested tables.
- [Comparison: Rust's own proposals](/docs/comparisons/rust-language-proposals): The context as a
  dictionary of providers, and the cycles it cannot yet express.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
