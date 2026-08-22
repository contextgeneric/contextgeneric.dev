---
sidebar_label: 'DefaultImpls1'
---

# `DefaultImpls1`

Resolving a namespace default keyed on a component *and* a type.

:::info

### Generated machinery

**You do not implement `DefaultImpls1`.**
[`#[default_impl(...)]`](../attributes/default_impl.md) emits the impls and a `for … in` statement inside
[`delegate_components!`](../macros/delegate_components.md) consumes them. You name the trait in those two
places only; this page explains what each of them emits, and the positional rule that is easy to get
backwards.

:::

## Overview

[`DefaultNamespace`](./default_namespace.md) keys a default on the component alone: one component, one
default provider. `DefaultImpls1` adds a second type to the key, which is what a **per-type default**
needs — the same component resolving differently for `String` than for `u64`:

```rust
pub trait DefaultImpls1<T, Components> {
    type Delegate;
}
```

There is no method and no data. Resolving a default *is* projecting `Delegate` from the matching impl,
exactly as with [`DelegateComponent`](./delegate_component.md).

Three traits exist rather than one variadic trait because each fixes the key's arity at the type level,
which is what lets the projection `<Key as Trait<…, Delegate = Provider>>` resolve cleanly.
[`DefaultImpls2`](./default_impls2.md) is the two-type form.

:::warning

**The parameter names are misleading about which position holds what.** For this trait and its sibling,
**`Self` is the instance type and the component name is a leading parameter** — the opposite of
[`DefaultNamespace`](./default_namespace.md). The [next section](#the-one-thing-to-get-right) works it
out.

:::

## Usage

**It is not in the prelude.** Import it from `cgp::core::component`, and you will need to, because the
generated impl references the trait by name:

```rust
use cgp::core::component::DefaultImpls1;
```

**You name it in two places**: the target of a `for … in` loop inside
[`delegate_components!`](../macros/delegate_components.md), and the
[`#[default_impl(...)]`](../attributes/default_impl.md) attribute that registers a provider as a default.
The macros generate the impls and the forwarding.

### The one thing to get right

Registering `ShowString` as the `String` default for `ShowImplComponent` emits:

```rust
impl<Components> DefaultImpls1<ShowImplComponent, Components> for String {
    type Delegate = ShowString;
}
```

`Self` is `String`; the trait's `T` parameter is filled by `ShowImplComponent`.

The rule that governs this comes from the attribute rather than the trait, and once you have it the
positions stop being surprising. **`#[default_impl(Key in NamespacePath)]` makes `Key` the impl's
`Self`** and appends the table parameter to whatever `NamespacePath` names — so the leading arguments are
simply whatever you wrote inside the path. The same rule is why the `for … in` loop's bound reads
`T: DefaultImpls1<Component, App, Delegate = Provider>`, with the loop variable in the `Self` position.

## Examples

The whole chain, from a provider declaring itself a default to a context pulling it in and overriding one
entry:

```rust
use cgp::core::component::DefaultImpls1;
use cgp::prelude::*;
use core::fmt::Display;

#[cgp_component(ShowImpl)]
#[prefix(@test in DefaultNamespace)]
pub trait Show<T> {
    fn show(&self, value: &T) -> String;
}

#[cgp_impl(new ShowString)]
#[default_impl(String in DefaultImpls1<ShowImplComponent>)]
impl ShowImpl<String> {
    fn show(&self, value: &String) -> String {
        value.clone()
    }
}
```

Then the context:

```rust
pub struct App;

delegate_components! {
    App {
        namespace DefaultNamespace;

        for <T, Provider> in DefaultImpls1<ShowImplComponent> {
            @test.ShowImplComponent.T: Provider,
        }

        @test.ShowImplComponent.u64: ShowWithDisplay,   // overrides the inherited default
    }
}
```

**Environmental context, parameter-targeted** — `App` carries the wiring and the shown value is a
parameter. The loop wires every type with a registered default by projecting
`T: DefaultImpls1<ShowImplComponent, App, Delegate = Provider>`, and the direct `u64` line shadows
whatever the namespace would otherwise supply for that one type.

A whole namespace can also be the loop target:

```rust
cgp_namespace! {
    new DefaultShowComponents {
        [String, u64]: ShowWithDisplay,
    }
}
```

Pointing `for <T, Provider> in DefaultShowComponents { … }` at this wires the listed types through the
same projection.

## When to reach for it, and when not

**Use it for a per-type default, and reach for the simpler trait when the key is a component alone.**

- **[`DefaultNamespace`](./default_namespace.md)** when one component has one default. That is the common
  case, and it is what [`#[prefix(...)]`](../macros/cgp_namespace.md) registers into.
- **`DefaultImpls1`** when the same component resolves differently per type. This is what
  [`#[default_impl]`](../attributes/default_impl.md) is usually pointed at.
- **[`DefaultImpls2`](./default_impls2.md)** for a two-type key.
- **Define your own namespace trait** when you want a named table of your own;
  [`#[default_impl]`](../attributes/default_impl.md) accepts any path, so these three are conveniences
  rather than the only options.

**One constraint decides where a `#[default_impl]` may be written, and it is Rust's orphan rule rather
than anything CGP chose.** The emitted impl is `impl Namespace<..> for Key`, so a crate may register a
default when it owns either the namespace trait or the key type. For an unprefixed component the key is
the component's own marker, so a downstream crate owning the component can register into a foreign
namespace. For a [`#[prefix]`](../macros/cgp_namespace.md)-ed component the key is a path built from
`cgp`-owned types plus the marker, so the impl is orphan-legal **only in the namespace's own crate**.
Wiring that must live downstream goes in the namespace body of the crate that owns it instead.

## Under the hood

A `for <T, Provider> in DefaultImpls1<Component> { … }` loop emits a
[`DelegateComponent`](./delegate_component.md) impl whose `where` clause projects the default:

```rust
where T: DefaultImpls1<Component, App, Delegate = Provider>
```

Read it as: for each type `T` that has a default, wire that key to the projected `Provider`.

**Because the loop variables appear only in that bound and in the key, the key must mention `T`** —
otherwise the parameter is unconstrained and the compiler rejects the impl with `E0207`, which reads as a
puzzling error about a generic parameter rather than about the loop.

The registration side is the mirror. [`#[default_impl]`](../attributes/default_impl.md) emits an impl of
this trait for the key type, carrying **only** the parameters naming the key and provider plus the table
— never the provider's own `where` clause. A provider whose bounds come from `#[use_type]`, `#[uses]`,
`#[implicit]`, or `#[use_provider]` therefore registers cleanly, because those bounds stay on the
provider's impl and its [`IsProviderFor`](./is_provider_for.md), and are checked when a real context
resolves it.

## Common Mistakes

**It is not in the prelude.** Import from `cgp::core::component`.
[`DefaultNamespace`](./default_namespace.md) is.

**`Self` is the instance type, not the component.** The parameter names suggest otherwise. Read
`#[default_impl(Key in Path)]` as "`Key` becomes `Self`" and the positions follow.

**A `for … in` loop's key must mention the loop variable**, or the impl is rejected with `E0207`.

**`#[default_impl]` on a prefixed component is confined to the namespace's crate**, by the orphan rule.
Put downstream wiring in the namespace body instead.

**A default is a fallback, not an assignment.** A direct entry silently shadows it.

**The registration impl carries none of the provider's bounds**, which is deliberate — they are checked
where the provider is used rather than where it is registered.

## Related constructs

- [`DefaultNamespace`](./default_namespace.md) — the one-key form, and the common case.
- [`DefaultImpls2`](./default_impls2.md) — the two-type form.
- [`#[default_impl(...)]`](../attributes/default_impl.md) — the attribute that emits impls of this trait.
- [`cgp_namespace!`](../macros/cgp_namespace.md) — defines a namespace, and documents `#[prefix(...)]`.
- [`delegate_components!`](../macros/delegate_components.md) — carries the `for … in` loop.
- [`DelegateComponent`](./delegate_component.md) — what the loop ultimately writes.
- [`IsProviderFor`](./is_provider_for.md) — where a registered provider's real bounds are checked.
- [`RedirectLookup`](../providers/redirect_lookup.md) — the usual `Delegate` value.

The ideas behind it:

- [Namespaces](/docs/concepts/namespaces) — reusable, inheritable wiring tables and preset-style
  configuration.

## Source

- [`namespaces.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-component/src/namespaces.rs)
  — the three lookup traits
- The `#[default_impl]` attribute: [`attributes/default_impl/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/attributes/default_impl)
- Header and loop codegen: [`delegate_component/statement/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/delegate_component/statement)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
