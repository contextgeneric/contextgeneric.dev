---
title: 'RedirectLookup — look up a component by path'
description: 'The generated provider that answers a component by looking a type-level path up in a table, the mechanism behind namespaces and the open statement.'
sidebar_label: 'RedirectLookup'
sidebar_position: 8
---

# `RedirectLookup`

Route a component's lookup along a type-level path in a separate table. The mechanism behind namespaces
and `open`.

:::info

### Generated machinery

**You are not expected to write `RedirectLookup` entries.**
[`#[cgp_component]`](../macros/cgp_component.md) generates a `RedirectLookup` impl for every component,
and the [`#[prefix]`](../attributes/prefix.md) attribute and the `open` and `namespace` statements of
[`delegate_components!`](../macros/delegate_components.md) generate the delegation entries that target
it. This page explains what those generated entries *are*, so that an expansion or a wiring error naming
this provider is legible.

:::

## Overview

`RedirectLookup<Components, Path>` separates *which key* a component is looked up under from *which
table* answers it. The ordinary provider blanket impl looks a component up in the
[**context**](/docs/reference/glossary#context)'s own delegation table, keyed by the [component
marker](/docs/reference/glossary#component-marker), where the context is the type the implementation
runs against. `RedirectLookup` does the lookup differently: it consults the table `Components` keyed
by a type-level `Path`, then delegates to whatever provider that entry holds. This indirection lets
one component's resolution be redirected to a different key in a different table, which is the basis
for organizing wiring into namespaces.

The redirection makes namespaces work. A namespace groups a context's components under a path
prefix so several related components can be wired in one place and addressed by a shared path.
`RedirectLookup` turns a prefixed path back into a concrete provider: the namespace machinery sets a
component's delegate to a `RedirectLookup` carrying the path under which the real provider was
registered, so a lookup of the component follows that path into the table and lands on the intended
provider. The `open` statement of [`delegate_components!`](../macros/delegate_components.md) is a
lightweight special case of the same redirection.

`RedirectLookup` is not written by hand. Every `#[cgp_component]` generates its impl, and the namespace
attributes generate the delegation entries whose delegate is a `RedirectLookup`. Reading those
generated entries is where this provider appears. Like every CGP provider, it carries no runtime value.

## Usage

`RedirectLookup` is in the prelude, but you do not name it directly. It appears where the namespace and
`open` machinery generate it. A component registers under a path with the
[`#[prefix(@path in Namespace)]`](../attributes/prefix.md) attribute, and a context that joins the
namespace binds the component's provider at that path, in its own table:

```rust
delegate_components! {
    App {
        namespace DefaultNamespace;

        @app.GreeterComponent: GreetHello,
    }
}
```

The path is a [`PathCons`](../types/path_cons.md) chain of [`Symbol!`](../macros/symbol.md) segments
and a component marker, written with the `@`-path syntax [`Path!`](../macros/path.md) also uses. The
declared struct is `RedirectLookup<Key, Components>`, but every generated impl and entry passes the
table first and the path second, as `RedirectLookup<Components, Path>`, so read the parameter names
as swapped.

## Examples

A component registers itself under `@app` in `DefaultNamespace`, and a context binds its provider at
that path:

```rust
use cgp::prelude::*;

#[cgp_component(Greeter)]
#[prefix(@app in DefaultNamespace)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self) -> String {
        "hello".into()
    }
}

pub struct App;

delegate_components! {
    App {
        namespace DefaultNamespace;

        @app.GreeterComponent: GreetHello,
    }
}

check_components! {
    App {
        GreeterComponent,
    }
}

pub fn demo() {
    assert_eq!(App.greet(), "hello");
}
```

`#[prefix]` makes `DefaultNamespace` route `GreeterComponent` to a `RedirectLookup` over the path
`@app.GreeterComponent`. The `namespace` statement makes `App` resolve its components through that
namespace, so `App.greet()` looks the path `@app.GreeterComponent` up in `App`'s own table and finds
`GreetHello`. The component marker is the last segment of the path rather than a key of its own. `App`
is an [environmental context](/docs/reference/glossary#environmental-context) with no fields.

## When to use it

**You do not reach for `RedirectLookup` directly.** Use the [`open` statement](../macros/delegate_components.md)
for per-type dispatch on one context, and a [namespace](../macros/cgp_namespace.md) for reusable,
inheritable wiring. Both generate the `RedirectLookup` entries for you. The
[namespaces](/docs/concepts/namespaces) concept explains when to reach for each.

Recognizing `RedirectLookup` in an expansion or a wiring error is the reason this page exists: a message
naming it means a lookup is being routed through a path, and the fix is usually a missing or misspelled
path entry rather than anything about the provider itself.

## Under the hood

[`#[cgp_component]`](../macros/cgp_component.md) generates a `RedirectLookup` impl of the provider trait
alongside the consumer blanket impl, the provider blanket impl, the component marker, and the
[`UseContext`](use_context.md) impl. The generated impl looks the path up in the table and forwards to
the resulting delegate. For a component such as

```rust
#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}
```

the macro generates this impl (shown with the macro's real placeholder identifiers):

```rust
impl<__Context__, __Components__, __Path__> Greeter<__Context__>
    for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Greeter<__Context__>,
{
    fn greet(__context__: &__Context__) -> String {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::greet(__context__)
    }
}
```

The mechanism is one [`DelegateComponent`](../traits/wiring/delegate_component.md) lookup keyed on
`__Path__` rather than on the component marker. `RedirectLookup<Components, Path>` implements
`Greeter` whenever `Components` maps `Path` to a delegate that itself implements `Greeter`, and the
method forwards to that delegate. When the consumer trait carries generic type parameters, the impl
first appends every one of them to the path with
[`ConcatPath`](../traits/formatting/concat_path.md), in declaration order, skipping lifetime and
const parameters. For `CanCalculateArea<Shape>`, `cargo cgp expand` shows, with the path resugared
as `Path!(@Shape)`:

```rust
impl<__Context__, Shape, __Components__, __Path__> AreaCalculator<__Context__, Shape>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@Shape)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@Shape)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Shape)>>::Output,
    >>::Delegate: AreaCalculator<__Context__, Shape>,
{
    fn area(__context__: &__Context__, shape: &Shape) -> f64 {
        <__Components__ as DelegateComponent<
            <__Path__ as ConcatPath<Path!(@Shape)>>::Output,
        >>::Delegate::area(__context__, shape)
    }
}
```

The lookup is still a single `DelegateComponent` query on the whole extended path, not a walk
segment by segment. A table answers a shorter prefix of the path because
[`delegate_components!`](../macros/delegate_components.md) generates entries generic over the
remaining segments, which is how the `open` statement's key forms work. As always, the impl is
paired with a matching [`IsProviderFor`](../traits/wiring/is_provider_for.md) impl.

The [`#[prefix(@path in Namespace)]`](../attributes/prefix.md) attribute populates the path side: it
generates a namespace impl whose delegate is `RedirectLookup<Components, Path>`, with the prefix path
joined onto the component marker, so resolving the component under that namespace follows the
prefixed path into the table.

## Common Mistakes

**A context that joins a namespace cannot also bind a registered component at its bare key.** Joining
`DefaultNamespace` already gives `App` an entry for `GreeterComponent`, the one that redirects to
`@app.GreeterComponent`, so adding `GreeterComponent: GreetHello` to the same table is a second impl
for the same key:

```text
error[E0119]: conflicting implementations of trait `IsProviderFor<GreeterComponent, _, _>` for type `App`
  --> src/main.rs:22:9
   |
20 |         namespace DefaultNamespace;
   |                   ---------------- first implementation here
21 |
22 |         GreeterComponent: GreetHello,
   |         ^^^^^^^^^^^^^^^^ conflicting implementation for `App`
```

Bind the provider at the registered path, `@app.GreeterComponent: GreetHello`, as the example does.

## Related constructs

- [`#[cgp_component]`](../macros/cgp_component.md) — generates a `RedirectLookup` impl for every
  component.
- [`cgp_namespace!`](../macros/cgp_namespace.md) — defines the namespaces whose entries target
  `RedirectLookup`.
- [`#[prefix]`](../attributes/prefix.md) — registers a component under a path that resolves through
  it.
- [`delegate_components!`](../macros/delegate_components.md) — the `open` and `namespace` statements that
  generate the redirect entries.
- [`DelegateComponent`](../traits/wiring/delegate_component.md) — the table the lookup reads.
- [`Path!`](../macros/path.md) and [`PathCons`](../types/path_cons.md) — the type-level path it
  looks up.
- [`UseContext`](use_context.md) — the other `#[cgp_component]`-generated provider, routing back to the
  context.

The ideas behind it:

- [Namespaces](/docs/concepts/namespaces) — reusable, inheritable wiring tables, which this provider
  implements.

## Source

- Struct:
  [`redirect_lookup.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-component/src/providers/redirect_lookup.rs)
- The generated impl:
  [`cgp_component/evaluated/to_redirect_lookup_impl.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/cgp_component/evaluated/to_redirect_lookup_impl.rs)
- The `#[prefix]` attribute that targets it:
  [`types/attributes/prefix.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/attributes/prefix.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
