---
sidebar_label: 'MutFieldGetter'
---

# `MutFieldGetter`

The provider-side mirror of `HasFieldMut` — wired field access that can mutate.

:::info

### Generated machinery

**You are not expected to implement `MutFieldGetter`.**
[`#[cgp_getter]`](../macros/cgp_getter.md) generates it alongside its supertrait, and
[`UseField`](../providers/use_field.md) satisfies both. What you write is the wiring entry; this page
explains what the mutable half of that entry provides. The one case for implementing it by hand is a provider whose mutable access is not a plain field read.

:::

## Overview

[`FieldGetter`](./field_getter.md) is field access in provider-trait shape, so that a context can choose
by wiring which field answers a getter. `MutFieldGetter` is the same thing with mutation: it hands back a
`&mut` rather than a `&`.

```rust
pub trait MutFieldGetter<Context, Tag>: FieldGetter<Context, Tag> {
    fn get_field_mut(context: &mut Context, tag: PhantomData<Tag>) -> &mut Self::Value;
}
```

It stands to [`FieldGetter`](./field_getter.md) exactly as [`HasFieldMut`](./has_field_mut.md) stands to
[`HasField`](./has_field.md): a **supertrait extension** rather than an alternative, with the field's
type inherited from the supertrait's `Value`. `Self` is the provider; `Context` is the type being
mutated.

Completing the square, the four traits divide on two axes — consumer versus provider, and read versus
write:

| | you bound against | you wire |
|---|---|---|
| read | [`HasField`](./has_field.md) | [`FieldGetter`](./field_getter.md) |
| write | [`HasFieldMut`](./has_field_mut.md) | `MutFieldGetter` |

## Using it

**It is in the prelude**, so `use cgp::prelude::*;` is enough.

As with its immutable sibling you rarely name the trait. What you write is a wiring entry naming a
provider that implements it — [`UseField`](../providers/use_field.md) does, so a getter component wired
to `UseField<Symbol!("counter")>` supports both the read and the write.

Writing an impl by hand is the escape hatch for a provider whose mutable access is not a plain field
read, and it is an ordinary trait impl on a marker type of your own. Implementing it means implementing
[`FieldGetter`](./field_getter.md) too, since that is the supertrait.

## Examples

A getter component whose mutable access is chosen by wiring:

```rust
use cgp::prelude::*;

#[cgp_getter(CounterGetter)]
pub trait HasCounter {
    fn counter(&self) -> &u64;
}

#[derive(HasField)]
pub struct App {
    pub request_count: u64,
}

delegate_components! {
    App {
        CounterGetterComponent: UseField<Symbol!("request_count")>,
    }
}
```

**Environmental context, self-targeted.** The getter reads `request_count` though it is named `counter`,
and because [`UseField`](../providers/use_field.md) implements `MutFieldGetter` as well, a provider that
holds `&mut App` can write through the same wiring.

## When to reach for it, and when not

**Wire it only when the mutable access must be chosen per context**, which is rarer than the read case
and rarer still than mutation generally.

- **Use an [`#[implicit]`](../attributes/implicit.md) argument** for a field the implementation reaches
  on its own context. The access rules on that page cover mutable access, and no provider is involved.
- **Use [`HasFieldMut`](./has_field_mut.md)** when the implementation should simply *require* mutable
  access rather than have it wired.
- **Use [`FieldGetter`](./field_getter.md)** when the wired access is read-only, which is the common case
  — requiring mutation narrows what a context can supply for no benefit.
- **Consider whether the context should be mutated at all.** Much CGP code keeps contexts immutable and
  threads state through handler outputs instead.

## Under the hood

:::note

### Advanced

This section is short, because the machinery is [`FieldGetter`](./field_getter.md#under-the-hood)'s.

:::

`MutFieldGetter` adds one method to its supertrait and no new resolution path: a provider that implements
[`FieldGetter`](./field_getter.md) and can also produce a `&mut` implements this too, and
[`UseField`](../providers/use_field.md) does for any context whose field is derived.

The route from a wired getter back to a context's own fields runs through
[`HasFieldMut`](./has_field_mut.md), whose `DerefMut` forwarding carries the `'static` bound on the
target that its immutable counterpart does not — so a context behind a smart pointer holding borrowed
data can satisfy the wired read and not the wired write.

## Gotchas

**`Self` is the provider, not the context.** The context is the first type parameter, and it arrives as
`&mut Context` in the method rather than as `&mut self`.

**Implementing it means implementing [`FieldGetter`](./field_getter.md).** It is a supertrait, so the
read half is not optional.

**`Value` lives on the supertrait**, so pin it there rather than redeclaring it.

**The `'static` bound behind `DerefMut` can bite.** A wired read that resolves and a wired write that
does not usually means the context is behind a smart pointer over borrowed data — see
[`HasFieldMut`](./has_field_mut.md#gotchas).

**Requiring it where a read would do narrows the contexts that fit**, since a provider holding `&Context`
cannot satisfy it.

## Related constructs

- [`FieldGetter`](./field_getter.md) — the supertrait, and where the wired form is explained in full.
- [`HasFieldMut`](./has_field_mut.md) — the consumer side you bound against.
- [`HasField`](./has_field.md) — the read half of the consumer side.
- [`UseField`](../providers/use_field.md) — the provider that implements both halves.
- [`#[cgp_getter]`](../macros/cgp_getter.md) — the macro that makes a getter a full component.
- [`#[implicit]`](../attributes/implicit.md) — the idiomatic way to reach a field.
- [`Symbol!`](../macros/symbol.md) and [`Index`](../types/index.md) — the tags that key a field.

The ideas behind it:

- [Consumer and provider traits](/docs/concepts/consumer-and-provider-traits) — the duality this trait is
  an instance of.

## Source

- [`has_field_mut.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/has_field_mut.rs)
  — `MutFieldGetter`, `HasFieldMut`, and the `DerefMut` forwarding

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
