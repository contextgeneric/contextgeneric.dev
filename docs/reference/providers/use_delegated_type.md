---
sidebar_label: 'UseDelegatedType'
---

# `UseDelegatedType`

Resolve an abstract type by looking its tag up in a table, instead of fixing it to one concrete type.

## Overview

`UseDelegatedType<Components>` is for the case where the concrete type an abstract type resolves to
should itself be decided by a table. The plain [`UseType<T>`](use_type.md) provider binds an abstract
type to one fixed `T`. But sometimes a single provider must answer several abstract-type components at
once, or route each type tag to a different concrete type chosen elsewhere, such as when a namespace or
a higher-order provider supplies a coherent bundle of types. Hand-writing one `UseType` entry per tag
would scatter that decision; `UseDelegatedType` concentrates it into one `Components` table the provider
consults. As always, the **context** (the type a capability runs against) is what points its type
components at the provider.

The mechanism is the same indirection [`UseDelegate`](use_delegate.md) provides for behavioral
components, lifted to the type level. Where `UseDelegate<Components>` dispatches a *method call* to
whichever provider `Components` maps the active tag to, `UseDelegatedType<Components>` dispatches a
*type resolution* to whichever concrete type `Components` maps the tag to. Both read an entry out of a
[`DelegateComponent`](../traits/delegate_component.md) table keyed by the tag, but one yields behavior
and the other yields a type. Like every CGP provider, it carries no runtime value.

## Usage

`UseDelegatedType` implements the built-in [`TypeProvider`](../components/has_type.md), so a
user-defined [`#[cgp_type]`](../macros/cgp_type.md) component is wired to it through its
[`WithProvider`](with_provider.md) alias, `WithDelegatedType`. Neither is in the prelude; import them
from `cgp::core::types`:

```rust
use cgp::core::types::WithDelegatedType;
```

`WithDelegatedType<Components>` takes one type parameter, the lookup table, and appears as the value of
one or more abstract-type components' wiring entries:

```rust
delegate_components! {
    App {
        [
            ScalarTypeProviderComponent,
            IndexTypeProviderComponent,
        ]: WithDelegatedType<AppTypes>,
    }
}
```

The `Components` parameter is a type that maps each type tag to a concrete type, built as an ordinary
[`delegate_components!`](../macros/delegate_components.md) table. Any tag the table has no entry for is
simply unresolved, and the context does not implement `HasType` for it. `WithDelegatedType<Components>`
is [`WithProvider<UseDelegatedType<Components>>`](with_provider.md), and it is the form you wire, since
the bare `UseDelegatedType` provides the foundational `TypeProvider` rather than a named component's
provider trait.

## Examples

A typical use defines a lookup table mapping type tags to concrete types and wires a context's type
components through it. The table is an ordinary type carrying `DelegateComponent` entries:

```rust
use cgp::prelude::*;
use cgp::core::types::WithDelegatedType; // not in the prelude

#[cgp_type]
pub trait HasScalarType {
    type Scalar;
}

#[cgp_type]
pub trait HasIndexType {
    type Index;
}

pub struct App;
pub struct AppTypes;

delegate_components! {
    AppTypes {
        ScalarTypeProviderComponent: f64,
        IndexTypeProviderComponent: usize,
    }
}

delegate_components! {
    App {
        [
            ScalarTypeProviderComponent,
            IndexTypeProviderComponent,
        ]: WithDelegatedType<AppTypes>,
    }
}
```

`App` routes both its scalar and index type components through `WithDelegatedType<AppTypes>`. When the
wiring asks for `App`'s `Scalar`, the provider looks `ScalarTypeProviderComponent` up in `AppTypes` and
finds `f64`; for `Index` it finds `usize`. One provider entry on `App` answers two abstract types, with
the concrete choices held in one place in `AppTypes` where they can be reused, swapped, or supplied by
a namespace.

## When to reach for it, and when not

**Reach for `UseDelegatedType` when several abstract types should be answered from one shared table**,
so a context points its type components at the table rather than fixing each one at the wiring site.
This is what makes a coherent bundle of types reusable across contexts or supplied by a namespace.

Prefer [`UseType<T>`](use_type.md) for the common case of fixing one abstract type to one concrete
type, which needs no table.

## Under the hood

`UseDelegatedType<Components>` implements [`TypeProvider`](../components/has_type.md) by looking the
type tag up in `Components` and reporting the delegate it finds as the abstract type:

```rust
#[cgp_provider(TypeProviderComponent)]
impl<Context, Tag, Components, Type> TypeProvider<Context, Tag> for UseDelegatedType<Components>
where
    Components: DelegateComponent<Tag, Delegate = Type>,
{
    type Type = Type;
}
```

The `where` clause is the whole of the behavior. `Components: DelegateComponent<Tag, Delegate = Type>`
reads the entry stored at key `Tag` in the table, and the impl sets the abstract `Type` to that
delegate. Because the lookup is keyed by `Tag`, one `UseDelegatedType<Components>` answers as many
distinct type tags as `Components` has entries, each resolving to its own concrete type. If the table
has no entry for a tag, the `DelegateComponent` bound is unsatisfied and the context does not implement
`HasType` for that tag. Contrast [`UseType<T>`](use_type.md), whose impl is unconditional and always
reports the single type `T`: `UseDelegatedType` adds exactly one level of indirection, the
`DelegateComponent` lookup.

## Related constructs

- [`UseType`](use_type.md) — the simpler sibling that fixes an abstract type to one concrete type.
- [`UseDelegate`](use_delegate.md) — the behavioral counterpart that dispatches a method call through
  the same kind of table.
- [`DelegateComponent`](../traits/delegate_component.md) — the table it reads.
- [`HasType` / `TypeProvider`](../components/has_type.md) — the abstract-type component it answers for.
- [`WithProvider`](with_provider.md) — the adapter behind the `WithDelegatedType` alias.
- [`#[cgp_type]`](../macros/cgp_type.md) — defines the abstract-type components it resolves.

The ideas behind it:

- [Abstract types](/docs/concepts/abstract-types) — the associated types a context chooses for itself,
  here from a shared table.

## Source

- Struct, `WithDelegatedType` alias, and the `TypeProvider` impl:
  [`use_delegated_type.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-type/src/impls/use_delegated_type.rs)
- The `HasType`/`TypeProvider` traits:
  [`has_type.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-type/src/traits/has_type.rs);
  `DelegateComponent`:
  [`delegate_component.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-component/src/traits/delegate_component.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
