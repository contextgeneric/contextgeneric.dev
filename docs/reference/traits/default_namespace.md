---
sidebar_label: 'DefaultNamespace'
---

# `DefaultNamespace`

Resolving a namespace's default provider for a component.

## What it's for

A [namespace](/docs/concepts/namespaces) is a reusable table of default wirings that a **context** — the
type the capability runs against — can opt into and then selectively override. Resolving one of those
defaults means asking: *for this component, what does the namespace delegate to?*

`DefaultNamespace` is what answers that:

```rust
pub trait DefaultNamespace<Components> {
    type Delegate;
}
```

`Self` is the component being looked up, `Components` is the table the lookup runs against — threaded
through so one key can resolve differently per context — and `Delegate` is the resolved provider. There
is no method and no data; resolving a default *is* projecting `Delegate` from the matching impl, exactly
as with [`DelegateComponent`](./delegate_component.md).

It is the simplest of three lookup traits, keyed on the component alone.
[`DefaultImpls1`](./default_impls1.md) adds one further type for a *per-type* default, and
[`DefaultImpls2`](./default_impls2.md) does the same under a pair.

**You name it in one place**: the `namespace` header of a wiring block. The macros generate the impls and
the forwarding, so this page is mostly about reading what they generate.

## Using it

**It is in the prelude**, so `use cgp::prelude::*;` is enough — unlike its two siblings, which are not.

A context joins a namespace with a header inside
[`delegate_components!`](../macros/delegate_components.md):

```rust
delegate_components! {
    App {
        namespace DefaultNamespace;
    }
}
```

after which every lookup `App` does not wire directly forwards through the namespace. A component
registers into one with the [`#[prefix(...)]`](../macros/cgp_namespace.md) attribute on its
`#[cgp_component]` trait.

## Examples

A component registered into the namespace, and a context joining it:

```rust
use cgp::prelude::*;

#[cgp_component(ShowImpl)]
#[prefix(@test in DefaultNamespace)]
pub trait Show<T> {
    fn show(&self, value: &T) -> String;
}

pub struct App;

delegate_components! {
    App {
        namespace DefaultNamespace;

        @test.ShowImplComponent.u64: ShowWithDisplay,   // overrides the inherited default
    }
}
```

**Environmental context, self-targeted.** The header forwards `App`'s unwired lookups through the
namespace, and the direct entry shadows whatever the namespace would otherwise supply for that one key —
the inheritance-with-override shape presets rely on.

## When to reach for it, and when not

**Write [`cgp_namespace!`](../macros/cgp_namespace.md) and the wiring statements; name this trait only
where the syntax requires it.** It appears in a `namespace` header and nowhere else in ordinary code.

- **Use `DefaultNamespace`** for a namespace whose defaults are per component, which is the common case
  and what [`#[prefix(...)]`](../macros/cgp_namespace.md) registers into.
- **Use [`DefaultImpls1`](./default_impls1.md)** for a per-type default, where one component resolves
  differently per type.
- **Define your own namespace trait instead** when you want a named table of your own;
  [`cgp_namespace!`](../macros/cgp_namespace.md) generates one, so this is a convenience rather than the
  only option.
- **Do not reach for a namespace at all** until the top-level wiring is long enough to be a problem. A
  namespace buys reuse and inheritance at the cost of one more indirection to follow when reading, and a
  short [`delegate_components!`](../macros/delegate_components.md) block needs neither.

## Under the hood

:::note

### Advanced

This section shows what a `namespace` header generates, and why an override does not conflict with it.

:::

A `namespace N;` header does not emit one entry — it emits a **blanket**
[`DelegateComponent`](./delegate_component.md) impl on the context that forwards every key through the
namespace:

```rust
impl<Key, Value> DelegateComponent<Key> for App
where
    Key: N<App, Delegate = Value>,
{
    type Delegate = Value;
}
```

paired with the matching [`IsProviderFor`](./is_provider_for.md) forwarding so dependencies stay
diagnosable.

**That blanket is what makes override work.** A directly-wired entry is a *concrete* impl for one key,
and a concrete impl is more specific than the blanket, so it resolves first — shadowing the inherited
default for that key and leaving the rest untouched. Two *concrete* entries for one key would conflict; a
concrete entry against a blanket does not.

Inheritance composes on top. A namespace declared `new Child: Parent { … }` emits a blanket impl
forwarding any key the parent resolves, so the child resolves everything the parent does plus its own
entries — and a context's direct entry still shadows either. All of it is projections, resolved at
compile time, with nothing at run time.

## Gotchas

**A namespace default is a fallback, not an assignment.** It resolves only for keys the context does not
wire directly, which is the intent, and it means a stray direct entry can silently shadow a default you
expected to apply.

**`Self` is the component here**, as you would expect — but **not** in
[`DefaultImpls1`](./default_impls1.md) and [`DefaultImpls2`](./default_impls2.md), where the instance
type takes the `Self` position instead. The inconsistency is the family's sharpest edge.

**It is in the prelude while its two siblings are not.** They come from `cgp::core::component`.

**There is no method.** Resolving a default is a type projection.

**Registering into a foreign namespace is bound by the orphan rule** — see
[`DefaultImpls1`](./default_impls1.md#when-to-reach-for-it-and-when-not), where the
[`#[default_impl]`](../attributes/default_impl.md) attribute's placement constraint is worked out.

## Related constructs

- [`DefaultImpls1`](./default_impls1.md) and [`DefaultImpls2`](./default_impls2.md) — the per-type and
  per-pair variants.
- [`#[default_impl(...)]`](../attributes/default_impl.md) — the attribute that registers a provider as a
  default.
- [`cgp_namespace!`](../macros/cgp_namespace.md) — defines a namespace, and documents `#[prefix(...)]`.
- [`delegate_components!`](../macros/delegate_components.md) — carries the `namespace` header.
- [`DelegateComponent`](./delegate_component.md) — what a namespace header forwards *into*.
- [`IsProviderFor`](./is_provider_for.md) — forwarded alongside, so dependency errors stay readable.
- [`RedirectLookup`](../providers/redirect_lookup.md) — the usual `Delegate` value, re-routing along a
  path.
- [`Path!`](../macros/path.md) — the paths a prefixed key is built from.

The ideas behind it:

- [Namespaces](/docs/concepts/namespaces) — reusable, inheritable wiring tables and preset-style
  configuration.

## Source

- [`namespaces.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-component/src/namespaces.rs)
  — the three lookup traits
- Header and loop codegen: [`delegate_component/statement/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/delegate_component/statement)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
