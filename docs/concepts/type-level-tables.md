---
title: 'Look up a provider in a type-level table'
sidebar_label: 'Type-level tables'
sidebar_position: 3
description: 'How a trait with an associated type works as a compile-time map, how a CGP call reaches its provider through one, and what such a table cannot do.'
---

# Type-level tables

[Context-Generic Programming](/docs/) (CGP) extends Rust with pluggable trait implementations, and a
type-level table records which one each context selects. The table is not a data structure in the
running program. It is a set of trait implementations that the compiler looks up while it checks
types, so choosing a provider costs nothing at runtime.

This page explains how such a table works, starting with plain Rust and no CGP names at all. It then
shows how a method call reaches its provider through the table, the other lookups CGP builds on the
same idea, who can write an entry, and what a table cannot do.

## A trait as a map

A trait whose only item is an associated type already works as a map in ordinary Rust. Each
implementation is one entry: the implementing type is the table, the trait's type parameter is the
key, and the associated type is the value. This table maps two shape types to the types that measure
them:

```rust
use core::marker::PhantomData;

pub trait HasEntry<Key> {
    type Value;
}

pub struct AreaTable;

pub struct Rectangle;
pub struct Circle;
pub struct RectangleArea;
pub struct CircleArea;

impl HasEntry<Rectangle> for AreaTable {
    type Value = RectangleArea;
}

impl HasEntry<Circle> for AreaTable {
    type Value = CircleArea;
}

pub fn entry<Key>() -> PhantomData<<AreaTable as HasEntry<Key>>::Value>
where
    AreaTable: HasEntry<Key>,
{
    PhantomData
}
```

Reading an entry takes two steps. The bound `AreaTable: HasEntry<Key>` asks whether the table has an
entry for `Key`, and the projection `<AreaTable as HasEntry<Key>>::Value` names its value. So
`entry::<Rectangle>()` has the type `PhantomData<RectangleArea>`, and `entry::<Circle>()` has the type
`PhantomData<CircleArea>`. A key without an entry fails the bound, and the program does not compile.

Rust's [coherence](/docs/reference/glossary#coherence) rules make this a map rather than a list of
pairs. A second entry for the same key would be a second implementation of `HasEntry<Rectangle>`
for `AreaTable`, and the compiler rejects it:

```rust
impl HasEntry<Rectangle> for AreaTable {
    type Value = RectangleArea;
}

// error[E0119]: conflicting implementations of trait `HasEntry<Rectangle>` for type `AreaTable`
impl HasEntry<Rectangle> for AreaTable {
    type Value = SquareArea;
}
```

Each key therefore has a single value. The lookup itself happens during type checking, so the
compiled program contains no table and never consults one.

## The wiring table

CGP's table trait has exactly this shape under another name,
[`DelegateComponent`](/docs/reference/traits/wiring/delegate_component). Its associated type is called
`Delegate`, the provider an entry delegates to:

```rust
pub trait DelegateComponent<Key: ?Sized> {
    type Delegate;
}
```

[`delegate_components!`](/docs/reference/macros/delegate_components) writes one implementation per
entry. The same `AreaTable` can be written with it, and read through `DelegateComponent` just as
before:

```rust
delegate_components! {
    new AreaTable {
        Rectangle: RectangleArea,
        Circle: CircleArea,
    }
}
```

The table that wires a context is the same trait, keyed by component markers instead of shapes. Take
the storage example from [Consumer and provider traits](./consumer-and-provider-traits.md): `App`
is an application type, an [environmental context](/docs/reference/glossary#environmental-context),
whose `CanFetchStorageObject` method fetches a stored object from Amazon S3. Its wiring entry
`StorageObjectFetcherComponent: FetchS3Object` is this implementation:

```rust
impl DelegateComponent<StorageObjectFetcherComponent> for App {
    type Delegate = FetchS3Object;
}
```

A call such as `app.fetch_storage_object("alice.png")` reaches `FetchS3Object` through two
[blanket implementations](/docs/reference/glossary#blanket-implementation) that
[`#[cgp_component]`](/docs/reference/macros/cgp_component) generates. The first gives a context the
consumer trait whenever it implements the provider trait for itself:

```rust
impl<Context> CanFetchStorageObject for Context
where
    Context: StorageObjectFetcher<Context>,
{
    fn fetch_storage_object(&self, object_id: &str) -> anyhow::Result<Vec<u8>> {
        Context::fetch_storage_object(self, object_id)
    }
}
```

The second reads the table. It gives any type with an entry for the component the provider trait of
whatever that entry names. Here it is with only its table bound, and with readable parameter names:

```rust
impl<Context, Table> StorageObjectFetcher<Context> for Table
where
    Table: DelegateComponent<StorageObjectFetcherComponent>,
    Table::Delegate: StorageObjectFetcher<Context>,
{
    fn fetch_storage_object(
        context: &Context,
        object_id: &str,
    ) -> anyhow::Result<Vec<u8>> {
        Table::Delegate::fetch_storage_object(context, object_id)
    }
}
```

Follow the call through both. The first implementation needs `App: StorageObjectFetcher<App>`. The
second can supply it, with `Table` and `Context` both standing for `App`: it reads `App`'s entry,
finds `FetchS3Object`, and needs `FetchS3Object: StorageObjectFetcher<App>`, which the provider's own
implementation provides. So `App` gains the provider trait through its table, and the consumer trait
through the first implementation. The context stays `App` throughout, which means the provider
receives the same application value the caller invoked the method on.

In a real expansion, the parameters are named `__Context__` and `__Provider__`, and the delegate is
spelled out as `<__Provider__ as DelegateComponent<StorageObjectFetcherComponent>>::Delegate`.
[`cargo cgp expand`](/docs/tooling/cargo-cgp/expand) shows the full expansion of a concrete component.

## The hidden requirement

The listing above leaves out one bound, and the real implementation has it. Beside its table bound,
the generated implementation requires the table to implement
[`IsProviderFor`](/docs/reference/traits/wiring/is_provider_for) for the component:

```rust
impl<Context, Provider> StorageObjectFetcher<Context> for Provider
where
    Provider: DelegateComponent<StorageObjectFetcherComponent>
        + IsProviderFor<StorageObjectFetcherComponent, Context, ()>,
    Provider::Delegate: StorageObjectFetcher<Context>,
{
    fn fetch_storage_object(
        context: &Context,
        object_id: &str,
    ) -> anyhow::Result<Vec<u8>> {
        Provider::Delegate::fetch_storage_object(context, object_id)
    }
}
```

`DelegateComponent` itself carries no such bound. Instead, `delegate_components!` writes an
`IsProviderFor` implementation beside each entry, which passes on the chosen provider's own
requirements. When one of those requirements is missing, the compiler can then name it, rather than
reporting only that a trait is not implemented. The same bound means that a `DelegateComponent`
implementation written by hand, without its `IsProviderFor` companion, does not wire the component.
[Checking your wiring](./check-traits.md) explains `IsProviderFor` and how a check uses it to find a
missing requirement.

## Tables keyed by other types

A table can be keyed by any type, so a provider can perform a second lookup of its own. Older CGP
code does this for a component with a type parameter through
[`UseDelegate`](/docs/reference/providers/use_delegate). Here `CanCalculateArea<Shape>` measures a
separate shape value, and `MyApp` is an environmental context with a
[parameter-targeted](/docs/reference/glossary#parameter-targeted-component) component. Its single
entry points to a nested table keyed by shape:

```rust
#[cgp_component(AreaCalculator)]
#[derive_delegate(UseDelegate<Shape>)]
pub trait CanCalculateArea<Shape> {
    fn area(&self, shape: &Shape) -> f64;
}

delegate_components! {
    MyApp {
        AreaCalculatorComponent:
            UseDelegate<new AreaCalculatorComponents {
                Rectangle: RectangleArea,
                Circle: CircleArea,
            }>,
    }
}
```

The `#[derive_delegate]` attribute gives `UseDelegate` an implementation that looks the shape up in
`AreaCalculatorComponents` and forwards to the provider it finds, so one wiring entry fans out to a
provider per shape. The handler family's `UseInputDelegate` performs the same lookup keyed by a
handler's input. Both are the legacy form, still found in existing code.

The [`open` statement](/docs/reference/macros/delegate_components#choosing-a-provider-per-type-the-open-statement)
replaces them. It needs no attribute on the component and keeps the per-shape entries in the
context's own table:

```rust
delegate_components! {
    MyApp {
        open AreaCalculatorComponent;

        @AreaCalculatorComponent.Rectangle: RectangleArea,
        @AreaCalculatorComponent.Circle: CircleArea,
    }
}
```

## Paths as keys

The `open` statement works by turning a component's type parameters into a path, and using the path
as the key. Its entry routes the component to
[`RedirectLookup`](/docs/reference/providers/redirect_lookup), a provider that every component
already has. `RedirectLookup` appends each of the component's type parameters to a type-level path
and looks the whole path up in the table, in one `DelegateComponent` query. The key
`@AreaCalculatorComponent.Rectangle` is that path, written with [`Path!`](/docs/reference/macros/path)
notation.

A path entry ends in a generic tail, so a key matches every longer path beneath it. That rule makes a
shorter key and a longer key under it two answers for the same lookup. With a converter component
that takes a source and a target type, this table is rejected:

```rust
delegate_components! {
    App {
        open ConverterComponent;

        // The shorter key matches every path beneath it, `@ConverterComponent.u8.u64` included.
        @ConverterComponent.u8: ConvertFromU8,
        @ConverterComponent.u8.u64: ConvertU8ToU64,
    }
}
// error[E0119]: conflicting implementations of trait
//               `DelegateComponent<PathCons<ConverterComponent, PathCons<u8, PathCons<u64, _>>>>`
//               for type `App`
```

The table has no rule that the most specific entry wins, so each lookup must have exactly one key that
matches it. A generic segment, such as `<Target> Target` in a key's last position, likewise matches
every type there.

## Nested tables and namespaces

An entry's value can be another table. An [aggregate provider](./aggregate-providers.md) is a table
declared with `new` and used as a provider: a context delegates a group of components to it, and each
lookup reads the context's entry, then the aggregate's entry for the same component. The context
stays the same throughout, so the provider at the end still receives the original context. The `->`
operator in `delegate_components!` similarly forwards one key to the same key's entry in another
table.

A [namespace](./namespaces.md) is a table written from the other side. In `DelegateComponent<Key>`,
the table is the implementing type, so only the crate that owns the table, usually the context, can
add an entry for a foreign key. A namespace entry instead implements the namespace trait for the key,
with the table left as a parameter. A component registered under a path with
[`#[prefix]`](/docs/reference/attributes/prefix), here `#[prefix(@app in AppNamespace)]`, produces an
implementation like this one, shown with a readable name for the table parameter. Its value routes
the lookup down the component's path:

```rust
impl<Table> AppNamespace<Table> for GreeterComponent {
    type Delegate = RedirectLookup<Table, Path!(@app.GreeterComponent)>;
}
```

Because this implementation is for the component marker, the crate that defines the component can
register its route in a namespace that it does not own, for every context that later joins.
Registered defaults follow the same orphan rule more narrowly:
[`#[default_impl]`](/docs/reference/attributes/default_impl) needs the crate to own the namespace, or a
local type before the table parameter, such as the key or a component named in the namespace path.

A context joins a namespace with `namespace AppNamespace;`, which adds one blanket entry that forwards
every key the namespace answers. A direct entry for a key the namespace already binds overlaps that
blanket entry and fails with `E0119`, which is why a shared namespace leaves open the paths its
contexts choose differently. A `for … in` loop in `delegate_components!` copies registered defaults
into a context's table.

## Other tables

The same shape appears elsewhere in CGP:

- **Fields:** [`HasField<Tag>`](/docs/reference/traits/field-access/has_field) is a table on a struct,
  keyed by a field's name as a type, whose value is the field's type, together with a method that
  reads the field.
- **Abstract types:** [`UseDelegatedType`](/docs/reference/providers/use_delegated_type) answers an
  abstract type from a `DelegateComponent` table keyed by the type's tag.
- **Checks:** [`CanUseComponent`](/docs/reference/traits/wiring/can_use_component) requires both an
  entry and a provider that can serve the context.

## What it costs

A table cannot express a preference. Every overlap is an error, whether between two entries, a short
path and a longer one, or a namespace's binding and a context's own entry. A shared table therefore
has to leave open the entries that contexts choose differently, and changing that design later can
affect every context that uses it.

Each lookup is a trait obligation, which the compiler proves rather than evaluates. A provider that
needs the very instance it is defining, such as a serializer for a recursive type that serializes its
tail through the context, overflows the trait solver with `E0275`. The usual workaround, and its
relation to the Rust project's work on cyclic impls, is in
[Rust's own proposals](/docs/comparisons/rust-language-proposals#the-context-as-a-dictionary).

An entry by itself checks nothing about its provider, so a missing requirement surfaces only when code
uses the component or a check forces it. When a check fails, the error can name path and symbol types
such as `PathCons<…>` and `Symbol<…>`. `cargo cgp expand` prints them in the `Path!` and `Symbol!`
notation used here, and [`cargo cgp check`](/docs/tooling/cargo-cgp/check) leads with the root cause
for the classes it recognizes, though it does not yet reshape every class.

The table is also fixed when the program compiles. It cannot be listed, inspected, or changed while the
program runs, and each lookup adds work for the compiler.

## Where to go next

These pages build on the table, explain its checks, and specify its constructs:

- [Consumer and provider traits](./consumer-and-provider-traits.md): The trait split the wiring table
  connects.
- [Checking your wiring](./check-traits.md): How `IsProviderFor` lets a check name a missing requirement.
- [Aggregate providers](./aggregate-providers.md) and [Namespaces](./namespaces.md): Tables shared
  between contexts.
- [`DelegateComponent`](/docs/reference/traits/wiring/delegate_component) and
  [`delegate_components!`](/docs/reference/macros/delegate_components): The table trait and the macro
  that writes it.
- [`RedirectLookup`](/docs/reference/providers/redirect_lookup),
  [`UseDelegate`](/docs/reference/providers/use_delegate), [`Path!`](/docs/reference/macros/path), and
  [`DefaultNamespace`](/docs/reference/traits/namespace/default_namespace): Path, nested, and namespace
  lookups.
- [Comparison: Rust's own proposals](/docs/comparisons/rust-language-proposals): The context as a
  dictionary of providers, and the cycles it cannot yet express.
- [Comparison: Dynamic dispatch](/docs/comparisons/dynamic-dispatch): A compile-time table beside a
  runtime vtable.
- In practice: [Dispatch layers](/docs/projects/cgp-examples/expression/architecture/dispatch-layers),
  the expression interpreter's provider written for a concrete context.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
