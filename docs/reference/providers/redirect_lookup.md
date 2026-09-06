---
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
table* answers it. The ordinary provider blanket impl looks a component up in the **context**'s own
delegation table, keyed by the component marker, where the context is the type the capability runs
against. `RedirectLookup` does the lookup differently: it consults the table `Components` keyed by a
type-level `Path`, then delegates to whatever provider that entry holds. This indirection lets
one component's resolution be redirected to a different key in a different table, which is the basis for
organizing wiring into namespaces.

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
`open` machinery generate it. A component is registered under a path with the
[`#[prefix(@path in Namespace)]`](../attributes/prefix.md) attribute, and a context joins a namespace
or opens a component for per-type dispatch:

```rust
delegate_components! {
    App {
        namespace DefaultNamespace;

        @bar.baz: TestProvider,
    }
}
```

A `RedirectLookup` later walks the path-keyed entry above (`@bar.baz`). The path itself is a
[`PathCons`](../types/path_cons.md) chain of [`Symbol!`](../macros/symbol.md) segments, most
easily written with [`Path!`](../macros/path.md).

## Examples

`RedirectLookup` appears in the delegate the namespace machinery generates, where a component is
registered under a path and reached through it:

```rust
use cgp::prelude::*;

pub struct App;

delegate_components! {
    App {
        namespace DefaultNamespace;

        @bar.baz: TestProvider,
    }
}
```

This registers `TestProvider` under the path `bar` then `baz` in `App`'s default namespace. When a
component is later resolved against `App` through that namespace, its delegate is a
`RedirectLookup<App, Path>` whose `Path` is the `PathCons` chain `bar`, then `baz`, then the component
marker. The lookup follows that path into `App`'s table, matches the entry above, and dispatches to
`TestProvider`. The component marker never keys the context directly; it is the tail of a path that
`RedirectLookup` walks. This is the indirection that lets namespaces organize wiring by path while still
resolving to ordinary providers.

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

The mechanism is one [`DelegateComponent`](../traits/wiring/delegate_component.md) lookup keyed on `__Path__`
rather than on the component marker. `RedirectLookup<Components, Path>` implements `Greeter` whenever
`Components` maps `Path` to a delegate that itself implements `Greeter`, and the method forwards to that
delegate. When the consumer trait carries generic type parameters, the impl additionally constrains
`Path` with [`ConcatPath`](../traits/formatting/concat_path.md) so the parameters are appended to the path before
the lookup, letting the redirected key encode the generic arguments. As always, the impl is paired with a matching
[`IsProviderFor`](../traits/wiring/is_provider_for.md) impl.

The [`#[prefix(@path in Namespace)]`](../attributes/prefix.md) attribute populates the path side: it
generates a namespace impl whose delegate is `RedirectLookup<Components, Path>`, with the prefix path
joined onto the component marker, so resolving the component under that namespace follows the
prefixed path into the table.

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
  walks.
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

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
