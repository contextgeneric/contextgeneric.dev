---
title: 'DelegateComponent — the wiring table entry'
description: 'The trait a wiring table is made of: one impl per entry, mapping a key such as a component name to the provider a context chose for it.'
sidebar_label: 'DelegateComponent'
sidebar_position: 1
---

# `DelegateComponent`

The per-context type-level table mapping a component key to its provider.

:::info

### Generated machinery

**You are not expected to write `DelegateComponent` impls.**
[`delegate_components!`](../../macros/delegate_components.md) emits one per entry of a wiring table,
and its `namespace` header emits the blanket impl that forwards every key through a namespace. What
you write is the table. This page explains what the table becomes, so that an expansion or a wiring
error naming this trait is legible. The one place you name the trait yourself is a bound that reads
an entry, as the [Usage](#usage) section shows.

:::

## Overview

Wiring a [**context**](/docs/reference/glossary#context), the type the implementation runs against,
means recording which provider it uses for each component. `DelegateComponent` is the trait that
record is made of, and one impl is one entry:

```rust
impl DelegateComponent<GreeterComponent> for App {
    type Delegate = GreetHello;
}
```

Read that as *"in `App`'s table, the entry at `GreeterComponent` is `GreetHello`."* The key and the
value are both types, and the lookup happens while the compiler type-checks, so the table costs
nothing at runtime and a wired call compiles to a direct one. Wiring is not a registry or a startup
step; it is a set of trait impls the compiler resolves.

**The key does not have to be a component name.** When it is, the entry gives the context the
component's provider trait. When it is any other type, such as a shape or a tag, the same trait is a
plain lookup table that a provider reads. One trait covers both, which is why it underlies ordinary
wiring and the inner dispatch tables alike.

## Definition

`DelegateComponent` carries a single associated type and no method:

```rust
pub trait DelegateComponent<Key: ?Sized> {
    type Delegate;
}
```

`Self` is the table: a context, or a provider bundle that owns its own entries. `Key` is the type
being looked up, and it is `?Sized` so that an unsized type can serve as a key. `Delegate` is the
value stored at that key: the provider, or a further table, that the entry resolves to. The trait
has no data, so the table exists only in the type system.

## Usage

The trait is in the prelude, so `use cgp::prelude::*;` is enough to name it.

### Setting an entry, and reading one

There are two operations, and both are ordinary Rust. **Setting** an entry is writing the impl,
which in practice means writing a [`delegate_components!`](../../macros/delegate_components.md)
line. **Reading** one is bounding on the trait and projecting the associated type:

```rust
pub fn provider_for<Table, Key>() -> PhantomData<<Table as DelegateComponent<Key>>::Delegate>
where
    Table: DelegateComponent<Key>,
{
    PhantomData
}
```

The bound is the lookup, and `<Table as DelegateComponent<Key>>::Delegate` is the value it finds.
Rust forbids two impls of one trait for the same `Self` and `Key`, so each key maps to exactly one
value: the structure is a map, and a duplicate entry is a
[coherence](/docs/reference/glossary#coherence) error rather than a silent overwrite.

### What can be a key

The key's type decides what an entry means, and three kinds occur:

- **A component name**, such as `GreeterComponent`. The provider blanket impl that
  [`#[cgp_component]`](../../macros/cgp_component.md) generates reads the entry, so the context gets
  the component's provider trait, and from it the consumer trait.
- **Any other type**, such as a shape or a tag. The table is a dispatch map with no provider trait
  attached, as the nested tables inside [`UseDelegate`](../../providers/use_delegate.md) are.
- **A type-level path**, a [`PathCons`](../../types/path_cons.md) list, which is how the `open`
  statement and namespaces key their entries. The macro emits a path entry with a generic tail, so
  it matches every longer path beneath it, and
  [`RedirectLookup`](../../providers/redirect_lookup.md) reads it with one lookup on the whole path.

### What owns a table

`Self` is not always a context. A [`delegate_components!`](../../macros/delegate_components.md)
block with a leading `new` declares a provider bundle that owns its own table, so several contexts
can delegate a group of components to it as one unit. A lookup is shallow: one read yields the
immediate `Delegate`, and chaining happens when that delegate is itself a table, as a bundle is.

## Examples

One component wired on a context, and a table keyed on shapes, each read back with the function from
[Usage](#usage):

```rust
use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self) -> String {
        "Hello!".to_owned()
    }
}

pub struct App;

delegate_components! {
    App {
        GreeterComponent: GreetHello,
    }
}

check_components! {
    App {
        GreeterComponent,
    }
}

pub struct Rectangle;
pub struct Circle;
pub struct RectangleArea;
pub struct CircleArea;

delegate_components! {
    new AreaComponents {
        Rectangle: RectangleArea,
        Circle: CircleArea,
    }
}

pub fn demo() {
    assert_eq!(App.greet(), "Hello!");

    // Each read resolves to the entry's value at compile time.
    let _: PhantomData<GreetHello> = provider_for::<App, GreeterComponent>();
    let _: PhantomData<RectangleArea> = provider_for::<AreaComponents, Rectangle>();
    let _: PhantomData<CircleArea> = provider_for::<AreaComponents, Circle>();
}
```

The `App` line is one `DelegateComponent` impl, and through it `App` gets the `Greeter` provider
trait and then `CanGreet`, so `App.greet()` compiles; swapping `GreetHello` for another provider
changes only that line. `AreaComponents` holds two entries keyed on shapes, with no provider trait
attached, which a [`UseDelegate`](../../providers/use_delegate.md) provider reads to pick an area
provider per shape. `App` is an
[environmental context](/docs/reference/glossary#environmental-context).

## When to use it

**Write [`delegate_components!`](../../macros/delegate_components.md), not `DelegateComponent`.**
The macro also emits the [`IsProviderFor`](./is_provider_for.md) forwarding impl beside each entry,
and the provider blanket impl requires both, so a hand-written `DelegateComponent` impl alone does
not wire the component at all. Name the trait yourself only to **read an entry in generic code**: a
dispatcher or a [higher-order provider](/docs/reference/glossary#higher-order-provider) that
resolves a key bounds on `DelegateComponent<Key>` and projects `Delegate`.

When a trait has exactly one implementation for a context, **implement the consumer trait directly
on the context** instead. A CGP consumer trait is an ordinary trait, so
`impl CanGreet for App { … }` is legal and needs no table entry. Wiring earns its place when the
implementation is one of several, is shared between contexts, or composes with a wrapper.

## Under the hood

`cargo cgp expand` shows what the `App` line of the example becomes: the entry, and beside it an
[`IsProviderFor`](./is_provider_for.md) impl that forwards the chosen provider's requirements:

```rust
impl DelegateComponent<GreeterComponent> for App {
    type Delegate = GreetHello;
}
impl<__Context__, __Params__> IsProviderFor<GreeterComponent, __Context__, __Params__>
for App
where
    GreetHello: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
```

The reader is the provider blanket impl that [`#[cgp_component]`](../../macros/cgp_component.md)
generates for `Greeter`. It implements the provider trait for any type that has both an entry and
the forwarding impl, and forwards each method to the entry's value:

```rust
impl<__Provider__, __Context__> Greeter<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<GreeterComponent>
        + IsProviderFor<GreeterComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        GreeterComponent,
    >>::Delegate: Greeter<__Context__>,
{
    fn greet(__context__: &__Context__) -> String {
        <__Provider__ as DelegateComponent<
            GreeterComponent,
        >>::Delegate::greet(__context__)
    }
}
```

The routing is the projection, and the call it forwards to is resolved statically. A context that
joins a namespace with a `namespace N;` header gets a blanket entry instead of one per key, which
reads each key's value from the namespace trait. For an `App` that joins a namespace `MyNamespace`
defined with [`cgp_namespace!`](../../macros/cgp_namespace.md), the header becomes:

```rust
impl<__Key__, __Value__> DelegateComponent<__Key__> for App
where
    __Key__: MyNamespace<App, Delegate = __Value__>,
{
    type Delegate = __Value__;
}
```

That is how a context inherits a whole table while still adding direct entries at the keys the
namespace leaves open.

## Common Mistakes

**A missing entry and an unsatisfied provider are different errors.** A context that never wires the
component fails a check on the table entry:

```rust
pub struct App;

check_components! {
    App {
        GreeterComponent,
    }
}
```

rustc names the missing entry in a help line:

```text
error[E0277]: the trait bound `App: CanUseComponent<GreeterComponent>` is not satisfied
...
help: the trait `DelegateComponent<GreeterComponent>` is not implemented for `App`
```

The fix is a wiring line. A wired component whose provider's dependencies are unmet fails through
[`IsProviderFor`](./is_provider_for.md) instead, and the fix is to supply the dependency.

**A hand-written entry does not wire the component.** An
`impl DelegateComponent<GreeterComponent> for App` written by hand, without the forwarding impl the
macro emits beside it, fails the provider blanket impl's `IsProviderFor` bound, so `App` never gets
the provider trait and a call fails:

```text
error[E0599]: the method `greet` exists for struct `App`, but its trait bounds were not satisfied
```

Write the entry with `delegate_components!`.

**Two entries for the same key conflict rather than override.** Wiring one component on one
context twice, to `GreetHello` and to a second provider `GreetHi`, is two impls of each generated
trait for one type:

```rust
delegate_components! {
    App {
        GreeterComponent: GreetHello,
        GreeterComponent: GreetHi,
    }
}
```

It fails twice, once for the forwarding impl and once for the entry:

```text
error[E0119]: conflicting implementations of trait `IsProviderFor<GreeterComponent, _, _>` for type `App`
...
error[E0119]: conflicting implementations of trait `DelegateComponent<GreeterComponent>` for type `App`
```

A direct entry for a key the context's namespace already binds conflicts the same way, with the
namespace's blanket impl, so a context adds entries only at keys the namespace leaves open.

**`Self` is not always a context.** A provider bundle declared with `new` owns a table too, and an
error naming one as the table does not mean something is being used as a context.

**The table is resolved at compile time, in full.** There is no runtime map to inspect and no way to
add an entry dynamically. If you are looking for a place to register something at startup, this is
not it.

## Related constructs

- [`delegate_components!`](../../macros/delegate_components.md): the macro that generates these
  impls; what you write instead of the trait.
- [`#[cgp_component]`](../../macros/cgp_component.md): generates the blanket impl that reads the
  table.
- [`IsProviderFor`](./is_provider_for.md): emitted beside each entry, and required with it.
- [`CanUseComponent`](./can_use_component.md): combines an entry and a valid provider into one
  assertion.
- [`UseDelegate`](../../providers/use_delegate.md): reads a table keyed on other types, for dispatch
  per type.
- [`RedirectLookup`](../../providers/redirect_lookup.md): reads a path-keyed entry; the mechanism
  behind `open` and namespaces.

The ideas behind it:

- [Consumer and provider traits](/docs/concepts/consumer-and-provider-traits): the split this table
  connects.
- [Bypassing coherence](/docs/concepts/coherence): why the choice is recorded per context at all.
- [Aggregate providers](/docs/concepts/aggregate-providers): a table owned by a provider rather than
  a context.

## Source

- [`traits/delegate_component.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-component/src/traits/delegate_component.rs)
- The entry codegen, in
  [`delegate_component/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/delegate_component),
  and the blanket impl that reads it, in
  [`cgp_component/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/cgp_component).

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
