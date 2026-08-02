---
sidebar_label: 'DefaultNamespace & DefaultImpls'
---

# `DefaultNamespace` & `DefaultImpls`

The namespace default-resolution traits, and `#[default_impl(...)]`.

## What it's for

A [namespace](/docs/concepts/namespaces) is a reusable table of default wirings that a **context** — the type
the capability runs against — can opt into and then selectively override. Resolving one of those defaults means
asking: *for this key, what does the namespace delegate to?*

These three traits are what answer that question. They are plain key-to-value lookups, one `Delegate` associated
type each, differing only in how many types take part in the key:

```rust
pub trait DefaultNamespace<Components> {
    type Delegate;
}

pub trait DefaultImpls1<T, Components> {
    type Delegate;
}

pub trait DefaultImpls2<T1, T2, Components> {
    type Delegate;
}
```

`DefaultNamespace` keys a default on the component alone. `DefaultImpls1` adds one further type — the shape for a
*per-type* default, where the same component resolves differently for `String` than for `u64`. `DefaultImpls2`
does the same under a pair.

Three traits rather than one variadic trait because each fixes the key's arity at the type level, which is what
lets the projection `<Key as Trait<…, Delegate = Provider>>` resolve cleanly.

**You name these only in two places**: the namespace header of a wiring block, and the target of a
`for … in` loop. The macros generate the impls and the forwarding — so this page is mostly about reading what
they generate, and about one detail that is genuinely easy to get backwards.

## Using it

`DefaultNamespace` is in the prelude. **`DefaultImpls1` and `DefaultImpls2` are not** — import them from
`cgp::core::component`, and you will need to, because the generated impl references the trait by name.

```rust
use cgp::core::component::DefaultImpls1;
```

`Self` is the key being looked up, `Components` is the table the lookup runs against — threaded through so one key
can resolve differently per context — and `Delegate` is the resolved value. There is no method and no data;
resolving a default *is* projecting `Delegate` from the matching impl, exactly as with
[`DelegateComponent`](./delegate_component.md).

### The one thing to get right

**The parameter names `T`, `T1`, and `T2` are misleading about which position holds what.** For
`DefaultNamespace` the `Self` key is the component name, as you would expect. For the two `DefaultImpls`
variants it is the other way round: **`Self` is the instance type, and the component name is a leading
parameter.** Registering `ShowString` as the `String` default for `ShowImplComponent` emits:

```rust
impl<Components> DefaultImpls1<ShowImplComponent, Components> for String {
    type Delegate = ShowString;
}
```

`Self` is `String`; the trait's `T` parameter is filled by `ShowImplComponent`.

The rule that actually governs this comes from the attribute rather than the trait, and once you have it the
positions stop being surprising. `#[default_impl(Key in NamespacePath)]` makes **`Key` the impl's `Self`** and
appends the table parameter to whatever `NamespacePath` names — so the leading arguments are simply whatever you
wrote inside the path. The same rule is why the `for … in` loop's bound reads
`T: DefaultImpls1<Component, App, Delegate = Provider>`, with the loop variable in the `Self` position.

### Registering a default

`#[default_impl(...)]` on a [`#[cgp_impl]`](../macros/cgp_impl.md) provider registers it as a default:

```rust
#[cgp_impl(new ShowString)]
#[default_impl(String in DefaultImpls1<ShowImplComponent>)]
impl ShowImpl<String> { /* … */ }
```

The registration impl carries only the parameters naming the key and provider, plus the table — **never the
provider's own `where` clause**. A provider whose bounds come from `#[use_type]`, `#[uses]`, `#[implicit]`, or
`#[use_provider]` registers cleanly, because those bounds stay on the provider's impl and its
[`IsProviderFor`](./is_provider_for.md) and are checked when a real context resolves it.

### Consuming defaults

A context joins a namespace with a header and pulls per-type defaults with a loop:

```rust
delegate_components! {
    App {
        namespace DefaultNamespace;

        for <T, Provider> in DefaultImpls1<ShowImplComponent> {
            @test.ShowImplComponent.T: Provider,
        }
    }
}
```

The header forwards `App`'s unwired lookups through the namespace; the loop wires each `T` that has a default. The
loop variable must appear in the key, or it is unconstrained and the impl is rejected.

## Examples

The whole chain, from a provider declaring itself a default to a context pulling it in and overriding one entry:

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

The loop wires every type with a registered default by projecting
`T: DefaultImpls1<ShowImplComponent, App, Delegate = Provider>`, and the direct `u64` line shadows whatever the
namespace would otherwise supply for that one type — the inheritance-with-override shape presets rely on.

A whole namespace can also be the loop target:

```rust
cgp_namespace! {
    new DefaultShowComponents {
        [String, u64]: ShowWithDisplay,
    }
}
```

Pointing `for <T, Provider> in DefaultShowComponents { … }` at this wires the listed types through the same
projection.

## When to reach for it, and when not

**Write [`cgp_namespace!`](../macros/cgp_namespace.md) and the wiring statements; name these traits only where the
syntax requires it.** They appear in a `namespace` header, in a `for … in` target, and in a `#[default_impl]`
attribute — and nowhere else in ordinary code.

- **Use `DefaultNamespace`** for a namespace whose defaults are per component, which is the common case and what
  [`#[prefix(...)]`](../macros/cgp_namespace.md) registers into.
- **Use `DefaultImpls1`** for a per-type default, where one component resolves differently per type. This is what
  `#[default_impl]` is usually pointed at.
- **Use `DefaultImpls2`** for a two-type key. It works, and it is the one member of the family that nothing in the
  library itself emits or consumes — a provided extension point rather than a construct the generated code relies
  on.
- **Define your own namespace trait instead** when you want a named table of your own;
  [`cgp_namespace!`](../macros/cgp_namespace.md) generates one, and `#[default_impl]` accepts any path, so these
  three are conveniences rather than the only options.
- **Do not reach for a namespace at all** until the top-level wiring is long enough to be a problem. A namespace
  buys reuse and inheritance at the cost of one more indirection to follow when reading, and a short
  [`delegate_components!`](../macros/delegate_components.md) block needs neither.

One constraint decides where a `#[default_impl]` may be *written*, and it is Rust's orphan rule rather than
anything CGP chose. The emitted impl is `impl Namespace<..> for Key`, so a crate may register a default when it
owns either the namespace trait or the key type. For an unprefixed component the key is the component's own
marker, so a downstream crate owning the component can register into a foreign namespace. For a
[`#[prefix]`](../macros/cgp_namespace.md)-ed component the key is a path built from `cgp`-owned types plus the
marker, so the impl is orphan-legal **only in the namespace's own crate**. Wiring that must live downstream goes
in the namespace body of the crate that owns it instead.

## Under the hood

:::note

### Advanced

This section shows what the header and the loop generate. You do not need it to use a namespace, but it explains
how a direct entry can shadow an inherited default without conflicting with it.

:::

A `namespace N;` header does not emit one entry — it emits a **blanket**
[`DelegateComponent`](./delegate_component.md) impl on the context that forwards every key through the namespace:

```rust
impl<Key, Value> DelegateComponent<Key> for App
where
    Key: N<App, Delegate = Value>,
{
    type Delegate = Value;
}
```

paired with the matching [`IsProviderFor`](./is_provider_for.md) forwarding so dependencies stay diagnosable.

**That blanket is what makes override work.** A directly-wired entry is a *concrete* impl for one key, and a
concrete impl is more specific than the blanket, so it resolves first — shadowing the inherited default for that
key and leaving the rest untouched. Two *concrete* entries for one key would conflict; a concrete entry against a
blanket does not.

A `for <T, Provider> in DefaultImpls1<Component> { … }` loop emits a `DelegateComponent` impl whose `where`
clause projects the default:

```rust
where T: DefaultImpls1<Component, App, Delegate = Provider>
```

Read it as: for each type `T` that has a default, wire that key to the projected `Provider`. Because the loop
variables appear only in that bound and in the key, the key must mention `T` — otherwise the parameter is
unconstrained and the compiler rejects the impl.

Inheritance composes on top. A namespace declared `new Child: Parent { … }` emits a blanket impl forwarding any
key the parent resolves, so the child resolves everything the parent does plus its own entries — and a context's
direct entry still shadows either. All of it is projections, resolved at compile time, with nothing at run time.

## Gotchas

**`DefaultImpls1` and `DefaultImpls2` are not in the prelude.** Import from `cgp::core::component`.
`DefaultNamespace` is.

**`Self` is the instance type for the `DefaultImpls` variants, not the component.** The parameter names suggest
otherwise. Read `#[default_impl(Key in Path)]` as "`Key` becomes `Self`" and the positions follow.

**A `for … in` loop's key must mention the loop variable.** Otherwise the parameter is unconstrained and the
impl is rejected with `E0207` — which reads as a puzzling error about a generic parameter rather than about the
loop.

**`#[default_impl]` on a prefixed component is confined to the namespace's crate.** The orphan rule decides this,
so a downstream default for a prefixed component is not something to work around — put the wiring in the
namespace body instead.

**A namespace default is a fallback, not an assignment.** It resolves only for keys the context does not wire
directly, which is the intent, and it means a stray direct entry can silently shadow a default you expected to
apply.

**`DefaultImpls2` has no user inside CGP.** It is reachable and tested, but nothing in the library emits or
consumes it — so there is no generated code to pattern-match against when using it.

## Related constructs

- [`cgp_namespace!`](../macros/cgp_namespace.md) — defines a namespace, and documents `#[prefix(...)]`.
- [`delegate_components!`](../macros/delegate_components.md) — carries the `namespace` header and the `for … in`
  loop that consume these traits.
- [`DelegateComponent`](./delegate_component.md) — what a namespace header forwards *into*.
- [`IsProviderFor`](./is_provider_for.md) — forwarded alongside, so dependency errors stay readable.
- [`RedirectLookup`](../providers/redirect_lookup.md) — the usual `Delegate` value, re-routing along a path.
- [`Path!`](../macros/path.md) — the paths a prefixed key is built from.
- [`#[cgp_impl]`](../macros/cgp_impl.md) — the host of the `#[default_impl]` attribute.

The ideas behind it:

- [Namespaces](/docs/concepts/namespaces) — reusable, inheritable wiring tables and preset-style configuration.

## Source

- The three traits: [`namespaces.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-component/src/namespaces.rs)
- The namespace macro: [`types/namespace/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/namespace)
- The `#[default_impl]` attribute: [`attributes/default_impl/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/attributes/default_impl)
- The header and loop codegen: [`delegate_component/statement/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/delegate_component/statement)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
