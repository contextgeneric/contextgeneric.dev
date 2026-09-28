---
title: 'DefaultNamespace — the default lookup table'
sidebar_label: 'DefaultNamespace'
sidebar_position: 1
description: 'The namespace trait a context joins with a namespace header, resolving each component''s default route; prefix registers components into it.'
---

# `DefaultNamespace`

Resolving a namespace's default provider for a component.

:::info

### Generated machinery

**You do not implement `DefaultNamespace`.** The [`#[prefix(...)]`](../../attributes/prefix.md)
attribute on a component emits its impls. You name the trait in two places, that attribute's
`in DefaultNamespace` and a `namespace` header inside
[`delegate_components!`](../../macros/delegate_components.md), and this page explains what they
generate, including why a direct entry can fill a path the namespace leaves open but cannot replace
one it binds.

:::

## Overview

A [namespace](/docs/concepts/namespaces) is a reusable table of default wirings that a **context** (the
type the method runs on) can opt into and then complete. Resolving one of those
defaults means asking: *for this component, what does the namespace delegate to?*

`DefaultNamespace` answers that. It is the simplest of three lookup traits, keyed on the component alone.
[`DefaultImpls1`](./default_impls1.md) adds one further type for a *per-type* default, and
[`DefaultImpls2`](./default_impls2.md) does the same under a pair.

**You name it in two places**: a component's `#[prefix(@path in DefaultNamespace)]` and the
`namespace` header of a wiring block. The macros generate the impls and the forwarding, so this page
is mostly about reading what they generate.

## Definition

`DefaultNamespace` carries a single associated type and nothing else:

```rust
pub trait DefaultNamespace<Components> {
    type Delegate;
}
```

`Self` is the component being looked up. `Components` is the table the lookup runs against, threaded
through as a parameter so one key can resolve differently for each context. `Delegate` is the
resolved value: the provider the key maps to. The trait has neither a method nor data, so resolving
a default projects `Delegate` from the matching impl, exactly as with
[`DelegateComponent`](../wiring/delegate_component.md).

## Usage

**It is in the prelude**, so `use cgp::prelude::*;` is enough, unlike its two siblings, which are not.

A context joins a namespace with a header inside
[`delegate_components!`](../../macros/delegate_components.md):

```rust
delegate_components! {
    App {
        namespace DefaultNamespace;
    }
}
```

after which every lookup `App` does not wire directly forwards through the namespace. A component
registers into one with the [`#[prefix(...)]`](../../attributes/prefix.md) attribute on its
`#[cgp_component]` trait.

## Examples

A component registered into the namespace, and a context joining it and filling the path it leaves
open:

```rust
use core::fmt::Display;
use cgp::prelude::*;

#[cgp_component(ShowImpl)]
#[prefix(@test in DefaultNamespace)]
pub trait Show<T> {
    fn show(&self, value: &T) -> String;
}

#[cgp_impl(new ShowWithDisplay)]
impl<T: Display> ShowImpl<T> {
    fn show(&self, value: &T) -> String {
        value.to_string()
    }
}

pub struct App;

delegate_components! {
    App {
        namespace DefaultNamespace;

        // The namespace routes the component to this path and binds nothing there.
        @test.ShowImplComponent.u64: ShowWithDisplay,
    }
}

check_components! {
    App {
        ShowImplComponent: u64,
    }
}

pub fn demo() {
    assert_eq!(App.show(&5u64), "5");
}
```

**[Environmental context](/docs/reference/glossary#environmental-context),
[parameter-targeted](/docs/reference/glossary#parameter-targeted-component).** `App` carries the
wiring and the shown value is a parameter. The header forwards `App`'s lookups through the
namespace. `#[prefix]` makes `DefaultNamespace` route `ShowImplComponent` to the path
`@test.ShowImplComponent` but leaves the paths beneath it unbound, so the direct entry supplies the
one for `u64`, and the check confirms it resolves.

## When to use it

**Write [`cgp_namespace!`](../../macros/cgp_namespace.md) and the wiring statements; name this trait
only where the syntax requires it.** It appears in a `#[prefix]` attribute and a `namespace` header
and nowhere else in ordinary code.

- **Use `DefaultNamespace`** for a namespace whose defaults are per component, which is the common case
  and what [`#[prefix(...)]`](../../attributes/prefix.md) registers into.
- **Use [`DefaultImpls1`](./default_impls1.md)** for a per-type default, where one component resolves
  differently per type.
- **Define your own namespace trait instead** when you want a named table of your own;
  [`cgp_namespace!`](../../macros/cgp_namespace.md) generates one, so this is a convenience rather than the
  only option.
- **Do not reach for a namespace at all** until the top-level wiring is long enough to be a problem. A
  namespace buys reuse and inheritance at the cost of one more indirection to follow when reading, and a
  short [`delegate_components!`](../../macros/delegate_components.md) block needs neither.

## Under the hood

A `namespace N;` header does not emit one entry. It emits a **blanket**
[`DelegateComponent`](../wiring/delegate_component.md) impl on the context that forwards every key
through the namespace, paired with the matching [`IsProviderFor`](../wiring/is_provider_for.md)
forwarding so dependencies stay diagnosable. `cargo cgp expand` on the example's `App` shows both:

```rust
impl<__Key__, __Value__> DelegateComponent<__Key__> for App
where
    __Key__: DefaultNamespace<App, Delegate = __Value__>,
{
    type Delegate = __Value__;
}
impl<
    __Key__,
    __Value__,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<__Key__, __Context__, __Params__> for App
where
    __Key__: DefaultNamespace<App, Delegate = __Value__>,
    __Value__: IsProviderFor<__Key__, __Context__, __Params__>,
{}
```

The namespace's own entries come from [`#[prefix(...)]`](../../attributes/prefix.md), which
implements the trait for the component with a [`RedirectLookup`](../../providers/redirect_lookup.md)
as the `Delegate`, re-routing the component to its path:

```rust
impl<__Components__> DefaultNamespace<__Components__> for ShowImplComponent {
    type Delegate = RedirectLookup<__Components__, Path!(@test.ShowImplComponent)>;
}
```

**That blanket is why a context cannot override a namespace entry.** A directly wired entry is a
second `DelegateComponent` impl for its key, and Rust lacks the specialization that would prefer one
impl over another. Where the namespace binds that key, the blanket covers it too, and the compiler
rejects the overlap with `E0119`. A direct entry compiles only for a key the blanket does not cover:
a path the namespace routes to but leaves unbound, like `@test.ShowImplComponent.u64` above.

Inheritance composes on top. A namespace declared `new Child: Parent { … }` emits a blanket impl
forwarding any key the parent resolves, so the child resolves everything the parent does plus its own
entries. The same rule holds at each level: a child cannot rebind a key its parent binds, and a
context cannot rebind a key either one binds. All of it is projections, resolved at compile time,
with nothing at run time.

## Common Mistakes

**A namespace entry cannot be overridden from the context.** A direct entry for a key the namespace
binds is rejected rather than preferred. Wiring the example's component by its bare name:

```rust
delegate_components! {
    App {
        namespace DefaultNamespace;

        ShowImplComponent: ShowWithDisplay,
    }
}
```

conflicts with the namespace's entry for `ShowImplComponent`, twice, once per generated trait:

```text
error[E0119]: conflicting implementations of trait `IsProviderFor<ShowImplComponent, _, _>` for type `App`
...
error[E0119]: conflicting implementations of trait `DelegateComponent<ShowImplComponent>` for type `App`
```

The form that works is the component's path, as the example wires `@test.ShowImplComponent.u64`. To
vary a choice between contexts, leave its path unbound in the namespace and wire it on each context,
or give the contexts different namespaces.

**`Self` is the component here**, as you would expect, but **not** in
[`DefaultImpls1`](./default_impls1.md) and [`DefaultImpls2`](./default_impls2.md), where the instance
type takes the `Self` position instead. The inconsistency is the family's sharpest edge.

**It is in the prelude while its two siblings are not.** They come from `cgp::core::component`.

**It lacks a method.** Resolving a default is a type projection.

**Registering into a foreign namespace is bound by the [orphan rule](/docs/reference/glossary#orphan-rule).** See
[`DefaultImpls1`](./default_impls1.md#when-to-use-it), where the
[`#[default_impl]`](../../attributes/default_impl.md) attribute's placement constraint is worked out.

## Related constructs

- [`DefaultImpls1`](./default_impls1.md) and [`DefaultImpls2`](./default_impls2.md): the per-type and
  per-pair variants.
- [`#[default_impl(...)]`](../../attributes/default_impl.md): the attribute that registers a provider as a
  default.
- [`cgp_namespace!`](../../macros/cgp_namespace.md): defines a namespace.
- [`#[prefix(...)]`](../../attributes/prefix.md): registers a component into a namespace under a path.
- [`delegate_components!`](../../macros/delegate_components.md): carries the `namespace` header.
- [`DelegateComponent`](../wiring/delegate_component.md): what a namespace header forwards *into*.
- [`IsProviderFor`](../wiring/is_provider_for.md): forwarded alongside, so dependency errors stay readable.
- [`RedirectLookup`](../../providers/redirect_lookup.md): the usual `Delegate` value, re-routing along a
  path.
- [`Path!`](../../macros/path.md): the paths a prefixed key is built from.

The ideas behind it:

- [Namespaces](/docs/concepts/namespaces): reusable, inheritable wiring tables and preset-style
  configuration.

## Source

- [`namespaces.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-component/src/namespaces.rs):
  the three lookup traits
- Header and loop codegen: [`delegate_component/statement/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/delegate_component/statement)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
