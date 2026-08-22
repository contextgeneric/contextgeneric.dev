---
sidebar_label: 'MapField'
---

# `MapField`

Reading through a field without forcing its type to be `'static`.

:::info

### Generated machinery

**You will not call `map_field` yourself.** It is a blanket impl that every
[`HasField`](./has_field.md) already has, and [`ChainGetters`](../providers/chain_getters.md) is what
uses it to descend into a nested context. This page explains the lifetime problem it exists to solve,
which is why reaching a nested field is a provider rather than two chained reads. The one case for calling it directly is writing a getter provider of your own that has to descend.

:::

## Overview

Chaining field reads looks like it should just work — read a field, then read a field of *that* —
and it does not, for a reason that is about lifetimes rather than about CGP. Writing
`context.get_field(..).get_field(..)` makes the intermediate `Value` outlive the borrow it came from,
which the compiler can only accept by requiring it to be `'static`.

`MapField` sidesteps that by taking a closure instead of returning the intermediate:

```rust
pub trait MapField<Tag>: HasField<Tag> {
    fn map_field<T>(
        &self,
        _tag: PhantomData<Tag>,
        mapper: impl for<'a> FnOnce(&'a Self::Value) -> &'a T,
    ) -> &T;
}
```

The higher-ranked bound is the whole trick: the closure is required to work for *any* lifetime, so the
compiler can pick the caller's rather than demanding `'static`.

**You will not normally call it.** [`ChainGetters`](../providers/chain_getters.md) is what uses
`map_field`, which is how a getter reaches a field on a nested context.

## Usage

**It is not in the prelude** — one of only two members of the `HasField` group that are not. Import it
from `cgp::core::field::traits`:

```rust
use cgp::core::field::traits::MapField;
```

It is a supertrait extension of [`HasField`](./has_field.md), so bounding on it gives you the plain read
as well. The `mapper` argument is an ordinary closure; what makes it unusual is the `for<'a>` binder,
which means a closure that captures a reference of some *specific* lifetime will not satisfy it.

**Nobody implements it.** It is a blanket impl for every `HasField` whose tag is `'static`, so every
field access gains `map_field` for free.

## Examples

Reaching a field of a field, which is the case the trait exists for:

```rust
use cgp::core::field::traits::MapField;
use cgp::prelude::*;

#[derive(HasField)]
pub struct Inner {
    pub name: String,
}

#[derive(HasField)]
pub struct Outer {
    pub inner: Inner,
}

let outer = Outer { inner: Inner { name: "Alice".to_owned() } };

let name: &String = outer.map_field(
    PhantomData::<Symbol!("inner")>,
    |inner| inner.get_field(PhantomData::<Symbol!("name")>),
);

assert_eq!(name, "Alice");
```

The closure receives the borrowed `Inner` and returns a borrow *derived from it*, so no intermediate
outlives its source and no `'static` bound appears anywhere.

In practice this is what a wiring line expresses instead:

```rust
delegate_components! {
    Outer {
        NameGetterComponent: ChainGetters<Symbol!("inner"), UseField<Symbol!("name")>>,
    }
}
```

which is the same descent, chosen at the wiring site rather than written at the call site.

## When to reach for it, and when not

**Reach for [`ChainGetters`](../providers/chain_getters.md) rather than this trait.** The provider is the
construct; `map_field` is the mechanism it is built from, and calling it by hand means writing at a call
site what the wiring could have decided.

- **Use [`ChainGetters`](../providers/chain_getters.md)** to reach a field on a nested context.
- **Use [`HasField`](./has_field.md)** for a field of the context itself, which is the common case and
  needs none of this.
- **Use an [`#[implicit]`](../attributes/implicit.md) argument** before either, since it covers every
  same-context read.
- **Reach for [`FieldMapper`](./field_mapper.md)** when the same descent must happen on the provider
  side, where the context is a type argument rather than `self`.

Call `map_field` directly only when you are writing machinery of the same kind — a new getter provider,
say — and even then the existing ones usually compose.

## Under the hood

:::note

### Advanced

This section explains why the blanket impl needs a `'static` tag and the value does not.

:::

`MapField` is a blanket impl for every [`HasField`](./has_field.md) whose **tag** is `'static`:

```rust
// for every Context: HasField<Tag>, where Tag: 'static
```

The bound is on the *tag*, not on the value — and that is the point. A tag is a
[`Symbol!`](../macros/symbol.md) or an [`Index<N>`](../types/index.md), both of which are `'static` by
construction, so the bound costs nothing. The value stays free, which is exactly what the naive chained
read could not achieve.

Inside, the implementation reads the field once and hands the borrow to the mapper. Because the mapper
is higher-ranked over `'a`, its return borrow is tied to the field borrow rather than to any fixed
lifetime, and the result can be returned to the caller.

[`ChainGetters`](../providers/chain_getters.md) composes descents by nesting these calls, one per
segment, so an arbitrarily deep path stays lifetime-correct with no `'static` anywhere in the chain.

## Gotchas

**It is not in the prelude.** Import from `cgp::core::field::traits`.
[`FieldMapper`](./field_mapper.md) is the other non-prelude member of the group; everything else in it
is in the prelude.

**The closure must be higher-ranked.** A closure that captures a reference with a particular lifetime
will not satisfy `for<'a>`, and the error talks about lifetime bounds rather than about the closure's
body — which is confusing the first time.

**The mapper must return a borrow *derived from* its argument.** Returning a reference to something else
in scope does not satisfy the binder, which is the safety the design buys.

**It cannot map to an owned value.** The signature returns `&T`. A getter that must produce a value uses
[`MRef`](../types/mref.md) instead.

**`MapField` is not [`MapFields`](./map_fields.md) and not [`MapType`](./map_type.md).** Three similar
names: this one is a lifetime helper for reading a nested field, `MapFields` applies a marker across a
type-level list, and `MapType` names one field's storage on a partial type.

## Related constructs

- [`FieldMapper`](./field_mapper.md) — the provider-side mirror of this trait.
- [`HasField`](./has_field.md) — the supertrait, and the plain read.
- [`ChainGetters`](../providers/chain_getters.md) — the provider that uses `map_field` to descend.
- [`#[implicit]`](../attributes/implicit.md) — the idiomatic way to read a field of the context itself.
- [`Symbol!`](../macros/symbol.md) and [`Index`](../types/index.md) — the tags, which are `'static` by
  construction.
- [`MRef`](../types/mref.md) — the return type for a getter that may produce rather than lend.
- [`MapFields`](./map_fields.md) and [`MapType`](./map_type.md) — the two similarly-named traits that do
  something else.

The ideas behind it:

- [Implicit arguments](/docs/concepts/implicit-arguments) — the ergonomic surface over field access.

## Source

- [`map_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/map_field.rs)
  — `MapField` and `FieldMapper`

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
