---
sidebar_label: '#[default_impl]'
sidebar_position: 7
---

# `#[default_impl(...)]`

Register a provider as a namespace's default for a key.

## Overview

A [namespace](/docs/concepts/namespaces) is a reusable table of default wirings that a **context**, the
type the capability runs against, can opt into and then selectively override. Ordinarily you write a
namespace's entries in its own body. `#[default_impl(...)]` lets a *provider* register itself instead,
at the point where it is defined:

```rust
#[cgp_impl(new ShowString)]
#[default_impl(String in DefaultImpls1<ShowImplComponent>)]
impl ShowImpl<String> { /* … */ }
```

Read that as: *`ShowString` is the default for `String`, in the `DefaultImpls1<ShowImplComponent>`
table.* A context that pulls that table in gets this provider without naming it.

Registering at the definition keeps the provider and its default together, so adding a new per-type
implementation is one place to edit rather than two.

## Usage

The attribute goes on a [`#[cgp_impl]`](../macros/cgp_impl.md) provider and takes one argument in two
parts, joined by the keyword `in`:

```rust
#[default_impl(Key in NamespacePath)]
```

**`Key` becomes the emitted impl's `Self`**, and `NamespacePath` names the lookup trait plus whatever
leading arguments you write inside it. The macro appends the table parameter for you. So the example above
emits:

```rust
impl<Components> DefaultImpls1<ShowImplComponent, Components> for String {
    type Delegate = ShowString;
}
```

That rule is worth internalizing, because the trait's own parameter names suggest the opposite
arrangement (see [`DefaultImpls1`](../traits/default_impls1.md#the-one-thing-to-get-right)).

**The path may name any trait**, not only the three CGP ships. A trait of your own with the right shape
works identically, which is why [`DefaultImpls2`](../traits/default_impls2.md) needed no new construct to
be usable.

The three built-in targets are [`DefaultNamespace`](../traits/default_namespace.md) for a component-only
key, [`DefaultImpls1`](../traits/default_impls1.md) for a per-type default (the usual choice), and
[`DefaultImpls2`](../traits/default_impls2.md) for a two-type key.

## Examples

The whole chain, from a provider declaring itself a default to a context pulling it in:

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

**Environmental context, parameter-targeted**: `App` carries the wiring and the shown value is a
parameter. The loop wires every type with a registered default, and the direct `u64` line shadows
whatever the namespace would otherwise supply for that one type.

## When to use it

**Reach for it when a provider is the natural default for its key and you want that recorded where the
provider is written.**

- **Write the entry in the namespace body instead** when the defaults belong together as a set, or when
  the provider is one of several candidates and none is obviously *the* default. A
  [`cgp_namespace!`](../macros/cgp_namespace.md) body reads as a table; scattered attributes do not.
- **Use it** when per-type defaults accumulate (a conversion, a formatter, a codec with one provider per
  type), because then the alternative is a namespace body that has to be edited every time a type is
  added.
- **Do not reach for a namespace at all** until the top-level wiring is long enough to be a problem.

**One constraint decides where the attribute may be written, and it is Rust's orphan rule rather than
anything CGP chose.** The emitted impl is `impl Namespace<..> for Key`, so a crate may register a default
when it owns either the namespace trait or the key type. For an unprefixed component the key is the
component's own marker, so a downstream crate owning the component can register into a foreign namespace.
For a [`#[prefix]`](../macros/cgp_namespace.md)-ed component the key is a path built from `cgp`-owned
types plus the marker, so the attribute is orphan-legal **only in the namespace's own crate**. Wiring
that must live downstream goes in the namespace body of the crate that owns it instead.

## Under the hood

The attribute emits a single impl of the named lookup trait, for the key type:

```rust
impl<Components> DefaultImpls1<ShowImplComponent, Components> for String {
    type Delegate = ShowString;
}
```

The `Components` parameter is the table the lookup runs against, appended by the macro and left generic
so one registration serves every context.

**The registration impl carries only the parameters naming the key and the provider, plus the table,
never the provider's own `where` clause.** That is deliberate, and it lets the attribute work with
ordinary providers: a provider whose bounds come from [`#[use_type]`](./use_type.md),
[`#[uses]`](./uses.md), [`#[implicit]`](./implicit.md), or [`#[use_provider]`](./use_provider.md)
registers cleanly, because those bounds stay on the provider's impl and its
[`IsProviderFor`](../traits/is_provider_for.md) and are checked when a real context resolves it.

Consumption is the mirror. A `for <T, Provider> in DefaultImpls1<Component> { … }` loop inside
[`delegate_components!`](../macros/delegate_components.md) emits a
[`DelegateComponent`](../traits/delegate_component.md) impl whose `where` clause projects the default:

```rust
where T: DefaultImpls1<Component, App, Delegate = Provider>
```

Because the loop variables appear only in that bound and in the key, **the key must mention them**, or
the parameter is unconstrained and the compiler rejects the impl with `E0207`.

## Formal grammar

The attribute argument is a key type, the keyword `in`, and a namespace path, in the Rust Reference's
[notation](https://doc.rust-lang.org/reference/notation.html):

```ebnf
DefaultImplArgs -> Type `in` TypePath
```

`Type` is the key that becomes the emitted impl's `Self`, and `TypePath` is the lookup trait with its
leading generic arguments written out. The table parameter is appended by the macro and must not be
given. The attribute takes exactly one such argument; it is neither comma-separated nor repeatable for
several tables on one provider.

## Common Mistakes

**`Key` becomes `Self`, not a parameter.** Read `#[default_impl(Key in Path)]` as "`Key` becomes `Self`"
and the trait's positions follow. The parameter names on
[`DefaultImpls1`](../traits/default_impls1.md) suggest otherwise.

**Do not write the table parameter.** The macro appends it; supplying it yourself makes the path's arity
wrong.

**On a prefixed component it is confined to the namespace's crate**, by the orphan rule. This is not
something to work around: put the wiring in the namespace body instead.

**The lookup trait must be imported.** The emitted impl names it, so
[`DefaultImpls1`](../traits/default_impls1.md) and [`DefaultImpls2`](../traits/default_impls2.md) need
`use cgp::core::component::…`; [`DefaultNamespace`](../traits/default_namespace.md) is in the prelude.

**It takes one argument and is not repeatable** for several tables on one provider.

**A registered default is a fallback, not an assignment.** A context's direct entry silently shadows it.

## Related constructs

- [`DefaultImpls1`](../traits/default_impls1.md) — the usual target, and where the positional rule is
  worked out.
- [`DefaultNamespace`](../traits/default_namespace.md) and
  [`DefaultImpls2`](../traits/default_impls2.md) — the component-only and two-type targets.
- [`cgp_namespace!`](../macros/cgp_namespace.md) — defines a namespace, and documents
  `#[prefix(...)]`.
- [`delegate_components!`](../macros/delegate_components.md) — carries the `namespace` header and the
  `for … in` loop that consume a registration.
- [`#[cgp_impl]`](../macros/cgp_impl.md) — the host this attribute goes on.
- [`IsProviderFor`](../traits/is_provider_for.md) — where a registered provider's real bounds are
  checked.
- [`RedirectLookup`](../providers/redirect_lookup.md) — the usual `Delegate` value.

The ideas behind it:

- [Namespaces](/docs/concepts/namespaces) — reusable, inheritable wiring tables and preset-style
  configuration.

## Source

- The attribute: [`attributes/default_impl/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/attributes/default_impl)
- The lookup traits: [`namespaces.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-component/src/namespaces.rs)
- Header and loop codegen: [`delegate_component/statement/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/delegate_component/statement)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
