---
sidebar_label: 'DefaultImpls2'
---

# `DefaultImpls2`

Resolving a namespace default keyed on a component and *two* types.

:::info

### Generated machinery

**You do not implement `DefaultImpls2`.**
[`#[default_impl(...)]`](../attributes/default_impl.md) emits the impls and a `for … in` statement
consumes them, exactly as for its one-type sibling. You name the trait in those two places only; this
page explains what they emit.

:::

## Overview

[`DefaultImpls1`](./default_impls1.md) keys a namespace default on a component plus one type, which is
what a per-type default needs. `DefaultImpls2` is the same idea under a **pair** of types:

```rust
pub trait DefaultImpls2<T1, T2, Components> {
    type Delegate;
}
```

There is no method and no data — resolving a default is projecting `Delegate` from the matching impl.

Three traits exist rather than one variadic trait because each fixes the key's arity at the type level,
which is what lets the projection resolve cleanly. This is the widest of the three.

:::warning

**Nothing inside CGP emits or consumes it.** It is reachable and tested, and it is a provided extension
point rather than a construct the generated code relies on — so there is no library-generated example to
pattern-match against. The [next section](#usage) shows what a use looks like.

:::

## Usage

**It is not in the prelude.** Import it from `cgp::core::component`:

```rust
use cgp::core::component::DefaultImpls2;
```

The positional rule is [`DefaultImpls1`](./default_impls1.md#the-one-thing-to-get-right)'s, extended by
one: **`Self` is the key type and the leading parameters are whatever you wrote inside the namespace
path.** So `#[default_impl(Key in DefaultImpls2<Component, Other>)]` emits

```rust
impl<Components> DefaultImpls2<Component, Other, Components> for Key {
    type Delegate = /* the provider */;
}
```

Registration is [`#[default_impl(...)]`](../attributes/default_impl.md), which accepts an arbitrary
namespace path and therefore needed no new construct to support this trait. Consumption is a `for … in`
loop inside [`delegate_components!`](../macros/delegate_components.md), whose bound projects `Delegate`
the same way the one-type form does.

## Examples

A two-type key registered and then consumed:

```rust
use cgp::core::component::DefaultImpls2;
use cgp::prelude::*;

#[cgp_impl(new ConvertStringToU64)]
#[default_impl(String in DefaultImpls2<ConverterComponent, u64>)]
impl Converter<u64> {
    fn convert(&self, value: &String) -> u64 {
        value.parse().unwrap_or_default()
    }
}
```

and a context pulling every registered pair in:

```rust
pub struct App;

delegate_components! {
    App {
        namespace DefaultNamespace;

        for <T, Provider> in DefaultImpls2<ConverterComponent, u64> {
            @test.ConverterComponent.T: Provider,
        }
    }
}
```

**Environmental context, parameter-targeted.** The loop variable must still appear in the key, exactly as
for the one-type form.

## When to reach for it, and when not

**Reach for it when a default genuinely depends on two types**, and prefer the narrower forms otherwise —
each extra key position is one more thing a reader has to hold.

- **[`DefaultNamespace`](./default_namespace.md)** when the key is the component alone. The common case.
- **[`DefaultImpls1`](./default_impls1.md)** when one further type decides the default. This is what
  `#[default_impl]` is usually pointed at.
- **`DefaultImpls2`** when two do — a conversion keyed on source *and* target, say.
- **Define your own namespace trait** when the shape does not fit.
  [`#[default_impl]`](../attributes/default_impl.md) accepts any path, so a trait of your own with
  whatever arity you need works identically. Given that nothing in the library uses this one, a
  purpose-named trait is often the clearer choice.

The orphan-rule constraint on where a registration may be *written* is the same as for the one-type form,
and is worked out on [its page](./default_impls1.md#when-to-reach-for-it-and-when-not).

## Under the hood

A `for … in` loop emits a [`DelegateComponent`](./delegate_component.md) impl whose `where` clause
projects the default:

```rust
where T: DefaultImpls2<Component, Other, App, Delegate = Provider>
```

The loop variables appear only in that bound and in the key, so **the key must mention them** — otherwise
the parameter is unconstrained and the compiler rejects the impl with `E0207`.

The registration impl carries only the parameters naming the key and provider plus the table, never the
provider's own `where` clause, so a provider with impl-side dependencies registers cleanly and its bounds
are checked when a real context resolves it.

Because nothing in the library emits this trait, there is no generated code to compare against — the
impls you see are the ones you or the attribute wrote.

## Common Mistakes

**Nothing inside CGP uses it.** There is no generated code to pattern-match against, and no worked
library example.

**It is not in the prelude.** Import from `cgp::core::component`.

**`Self` is the key type, not the component.** The parameter names `T1` and `T2` suggest otherwise. Read
`#[default_impl(Key in Path)]` as "`Key` becomes `Self`".

**A `for … in` loop's key must mention the loop variables**, or the impl is rejected with `E0207`.

**A default is a fallback, not an assignment.** A direct entry silently shadows it.

**Two keys are usually one key too many.** If the second type is fixed for a whole application, a
[`DefaultImpls1`](./default_impls1.md) default plus a wiring entry is easier to read.

## Related constructs

- [`DefaultImpls1`](./default_impls1.md) — the one-type form, and where the family's mechanics are
  worked out.
- [`DefaultNamespace`](./default_namespace.md) — the component-only form.
- [`#[default_impl(...)]`](../attributes/default_impl.md) — the attribute that emits impls of this trait.
- [`cgp_namespace!`](../macros/cgp_namespace.md) — defines a namespace.
- [`delegate_components!`](../macros/delegate_components.md) — carries the `for … in` loop.
- [`DelegateComponent`](./delegate_component.md) — what the loop ultimately writes.
- [`RedirectLookup`](../providers/redirect_lookup.md) — the usual `Delegate` value.

The ideas behind it:

- [Namespaces](/docs/concepts/namespaces) — reusable, inheritable wiring tables and preset-style
  configuration.

## Source

- [`namespaces.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-component/src/namespaces.rs)
  — the three lookup traits
- The `#[default_impl]` attribute: [`attributes/default_impl/`](https://github.com/contextgeneric/cgp/tree/main/crates/macros/cgp-macro-core/src/types/attributes/default_impl)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
