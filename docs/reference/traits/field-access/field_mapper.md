---
sidebar_label: 'FieldMapper'
sidebar_position: 6
---

# `FieldMapper`

The provider-side mirror of `MapField`.

:::info

### Generated machinery

**You will not call `map_field` yourself.** It is a blanket impl over every
[`FieldGetter`](./field_getter.md), and [`ChainGetters`](../../providers/chain_getters.md) is its only
caller in practice. This page explains the mechanism a chained getter is built from. The one case for calling it directly is writing a getter provider of your own that has to descend.

:::

## Overview

[`MapField`](./map_field.md) reads through a field without forcing its type to be `'static`, by taking a
higher-ranked closure instead of returning the intermediate borrow. `FieldMapper` is the same operation
in provider-trait shape, with the context as an explicit type argument rather than as `&self`. The
group's two-by-two shape holds here too:

| | you bound against | you wire |
|---|---|---|
| plain read | [`HasField`](./has_field.md) | [`FieldGetter`](./field_getter.md) |
| read through | [`MapField`](./map_field.md) | `FieldMapper` |

**This is the one [`ChainGetters`](../../providers/chain_getters.md) actually uses**, because a chained
getter is a provider composing other providers rather than a method on a concrete type.

## Definition

`FieldMapper<Context, Tag>` is the provider-trait mirror of [`MapField`](./map_field.md), with the context
as an explicit type argument instead of `&self`:

```rust
pub trait FieldMapper<Context, Tag>: FieldGetter<Context, Tag> {
    fn map_field<T>(
        context: &Context,
        _tag: PhantomData<Tag>,
        mapper: impl for<'a> FnOnce(&'a Self::Value) -> &'a T,
    ) -> &T;
}
```

`Self` is the provider and `Context` is the type being read from. It is a supertrait extension of
[`FieldGetter`](./field_getter.md), exactly as [`MapField`](./map_field.md) extends
[`HasField`](./has_field.md), so `Value` comes from the supertrait. `map_field` takes the context by
shared reference, the `PhantomData<Tag>` that names the field, and a `mapper` closure whose higher-ranked
`for<'a>` bound ties the returned `&T` to the field's own lifetime.

## Usage

**It is not in the prelude.** Import it from `cgp::core::field::traits`:

```rust
use cgp::core::field::traits::FieldMapper;
```

**Nobody implements it.** It is a blanket impl for every [`FieldGetter`](./field_getter.md) with the
getter and the tag `'static`, so every field-getter provider gains `map_field` for free, which makes
chaining composable without each provider opting in.

The `mapper` argument carries the same `for<'a>` binder as its consumer-side twin, and for the same
reason: it lets the returned borrow be tied to the field borrow rather than to a fixed lifetime.

## Examples

You meet it through a wiring line rather than a call. A getter that reaches a field on a nested context:

```rust
use cgp::prelude::*;

#[cgp_getter(NameGetter)]
pub trait HasName {
    fn name(&self) -> &String;
}

#[derive(HasField)]
pub struct Inner {
    pub name: String,
}

#[derive(HasField)]
pub struct Outer {
    pub inner: Inner,
}

delegate_components! {
    Outer {
        NameGetterComponent: ChainGetters<Symbol!("inner"), UseField<Symbol!("name")>>,
    }
}
```

**Environmental context, self-targeted.** [`ChainGetters`](../../providers/chain_getters.md) descends into
`inner` with `FieldMapper::map_field` and then applies the inner provider to what it finds, so
`outer.name()` resolves to `outer.inner.name` with no lifetime obligations leaking into either type.

Calling it directly is possible and rarely what you want:

```rust
use cgp::core::field::traits::FieldMapper;

let name = <UseContext as FieldMapper<Outer, Symbol!("inner")>>::map_field(
    &outer,
    PhantomData,
    |inner| inner.get_field(PhantomData::<Symbol!("name")>),
);
```

## When to use it

**Reach for [`ChainGetters`](../../providers/chain_getters.md) rather than this trait.** It is the construct;
`FieldMapper` is the mechanism underneath it.

- **Use [`ChainGetters`](../../providers/chain_getters.md)** to reach a field on a nested context by wiring.
- **Use [`FieldGetter`](./field_getter.md)** when the wired access is a plain one-level read.
- **Use [`MapField`](./map_field.md)** when the descent happens on `self` rather than on a wired context
  — the consumer-side twin.
- **Use an [`#[implicit]`](../../attributes/implicit.md) argument** before any of them for a field of the
  implementation's own context.

Name it directly only when writing a getter provider of your own that must descend, and check first
whether composing the existing ones does the job.

## Under the hood

`FieldMapper` is a blanket impl over every [`FieldGetter`](./field_getter.md), with two `'static` bounds
that its consumer-side twin needs only one of:

- the **tag** must be `'static`, which is free — a [`Symbol!`](../../macros/symbol.md) or an
  [`Index<N>`](../../types/index_type.md) is `'static` by construction;
- the **getter** — that is, `Self`, the provider — must be `'static`, which is also free, since a
  provider is a zero-sized marker type with no lifetime parameters in the ordinary case.

The field's *value* stays free of any bound, which is the whole point: it is the intermediate whose
lifetime the naive chained read could not express.

[`ChainGetters`](../../providers/chain_getters.md) composes descents by nesting `map_field` calls, one per
path segment, so an arbitrarily deep chain stays lifetime-correct. Because the impl is blanket, adding a
new getter provider makes it chainable with no extra work.

## Common Mistakes

**It is not in the prelude.** Import from `cgp::core::field::traits`.
[`MapField`](./map_field.md) is the other non-prelude member of the group.

**`Self` is the provider, not the context.** The context is the first type parameter and arrives as
`&Context` in the method.

**The closure must be higher-ranked.** A closure capturing a reference of a particular lifetime will not
satisfy `for<'a>`, and the error talks about lifetime bounds rather than about the closure's body.

**A provider with a non-`'static` parameter loses the blanket impl.** The bound on `Self` is free for an
ordinary zero-sized marker and not for one carrying a lifetime, which is a rare shape and a puzzling
failure when it happens.

**It cannot map to an owned value.** The signature returns `&T`; a getter that must produce a value uses
[`MRef`](../../types/mref.md).

## Related constructs

- [`MapField`](./map_field.md) — the consumer-side twin, and where the lifetime problem is explained in
  full.
- [`FieldGetter`](./field_getter.md) — the supertrait, and the plain wired read.
- [`ChainGetters`](../../providers/chain_getters.md) — the provider that uses this to descend.
- [`HasField`](./has_field.md) — the consumer side of field access.
- [`UseField`](../../providers/use_field.md) and [`UseContext`](../../providers/use_context.md) — the two
  providers a chain is usually built from.
- [`Symbol!`](../../macros/symbol.md) and [`Index`](../../types/index_type.md) — the tags, `'static` by construction.
- [`MRef`](../../types/mref.md) — the return type for a getter that may produce rather than lend.

The ideas behind it:

- [Consumer and provider traits](/docs/concepts/consumer-and-provider-traits) — the duality this trait is
  an instance of.

## Source

- [`map_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/map_field.rs)
  — `FieldMapper` and `MapField`

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
