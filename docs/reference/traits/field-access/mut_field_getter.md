---
title: 'MutFieldGetter — wired mutable field access'
description: 'The provider-side form of HasFieldMut: a getter with a &mut self method takes any MutFieldGetter provider, such as UseField, through WithProvider.'
sidebar_label: 'MutFieldGetter'
sidebar_position: 4
---

# `MutFieldGetter`

The provider-side form of `HasFieldMut`: wired field access that can mutate.

:::info

### Generated machinery

**You are not expected to name `MutFieldGetter` in wiring.** A getter component whose method takes
`&mut self` accepts any provider of it through
[`WithProvider`](../../providers/with_provider.md), and
[`UseField`](../../providers/use_field.md) implements it, so the wiring entry is `WithField<Tag>` as
for a read. The one case for implementing it yourself is a provider whose mutable access no existing
provider expresses.

:::

## Overview

[`FieldGetter`](./field_getter.md) is field access in provider shape, so that a
[**context**](/docs/reference/glossary#context), the type the method runs on, which supplies the
values it needs as its fields, can choose by wiring which field answers a getter. `MutFieldGetter`
is the same with mutation: it hands back a `&mut` rather than a `&`, for a getter whose method takes
`&mut self`. The four field-access traits divide on two axes:

| | you bound against | you wire |
|---|---|---|
| read | [`HasField`](./has_field.md) | [`FieldGetter`](./field_getter.md) |
| write | [`HasFieldMut`](./has_field_mut.md) | `MutFieldGetter` |

## Definition

`MutFieldGetter<Context, Tag>` extends [`FieldGetter`](./field_getter.md) with one method that
returns a mutable borrow:

```rust
pub trait MutFieldGetter<Context, Tag>: FieldGetter<Context, Tag> {
    fn get_field_mut(context: &mut Context, tag: PhantomData<Tag>) -> &mut Self::Value;
}
```

It stands to `FieldGetter` as [`HasFieldMut`](./has_field_mut.md) stands to `HasField`: a
[supertrait](/docs/reference/glossary#supertrait) extension, with the field's type inherited from
the supertrait's `Value`. `Self` is the provider and `Context` is the type being mutated, passed as
`&mut Context`.

## Usage

It is in the prelude, so `use cgp::prelude::*;` is enough. A getter whose method takes `&mut self`
and returns `&mut T` is wired the same way as a read, with `WithField<Tag>` from
`cgp::core::field::impls`:

```rust
delegate_components! {
    App {
        CounterGetterComponent: WithField<Symbol!("request_count")>,
    }
}
```

A bare `UseField<Tag>` entry works too, through the impl `#[cgp_getter]` emits for `UseField`
directly. Implementing the trait yourself means implementing `FieldGetter` as well, its supertrait.

## Examples

A mutable getter named `counter_mut` that reads the `request_count` field:

```rust
use cgp::prelude::*;
use cgp::core::field::impls::WithField;

#[cgp_getter(CounterGetter)]
pub trait HasCounter {
    fn counter_mut(&mut self) -> &mut u64;
}

#[derive(HasField)]
pub struct App {
    pub request_count: u64,
}

delegate_components! {
    App {
        CounterGetterComponent: WithField<Symbol!("request_count")>,
    }
}

check_components! {
    App {
        CounterGetterComponent,
    }
}

pub fn demo() {
    let mut app = App { request_count: 0 };
    *app.counter_mut() += 1;
    assert_eq!(app.request_count, 1);
}
```

`UseField` implements `MutFieldGetter` for any context whose field has
[`HasFieldMut`](./has_field_mut.md), which the derive always emits. `App` is an
[environmental context](/docs/reference/glossary#environmental-context), and the getter is
[self-targeted](/docs/reference/glossary#self-targeted-component).

## When to use it

**Wire mutable access only when a context must choose which field is mutated**, which is rarer than
the read case and rarer still than mutation generally.

- **An [`#[implicit]`](../../attributes/implicit.md) argument** reaches a field of the
  implementation's own context, mutably too, with no provider involved.
- **[`HasFieldMut`](./has_field_mut.md)** is the bound for an implementation that requires mutable
  access rather than having it wired.
- **[`FieldGetter`](./field_getter.md)** is the form for read-only wired access, the common case.

## Under the hood

For a one-method getter whose method takes `&mut self`, `#[cgp_getter]` bounds its `WithProvider`
impl on `MutFieldGetter` instead of `FieldGetter`. `cargo cgp expand` on the example shows:

```rust
impl<__Context__, __Provider__> CounterGetter<__Context__> for WithProvider<__Provider__>
where
    __Provider__: MutFieldGetter<__Context__, CounterGetterComponent, Value = u64>,
{
    fn counter_mut(__context__: &mut __Context__) -> &mut u64 {
        __Provider__::get_field_mut(
            __context__,
            ::core::marker::PhantomData::<CounterGetterComponent>,
        )
    }
}
```

`UseField<Tag>` implements `MutFieldGetter` for every tag it is asked under, through the context's
`HasFieldMut<Tag>`, and [`UseFieldRef`](../../providers/use_field_ref.md) implements it through
`AsMut`. `UseContext` has no `MutFieldGetter` impl, so a mutable getter cannot be wired through
`WithContext`.

## Common Mistakes

**`Self` is the provider, not the context.** The context is the first type parameter, and arrives as
`&mut Context` rather than as `&mut self`.

**Implementing it means implementing [`FieldGetter`](./field_getter.md).** It is a supertrait, so
the read half is not optional.

**The `'static` bound behind `DerefMut` can bite.** A context behind a smart pointer to borrowed
data satisfies the read and not the write; see [`HasFieldMut`](./has_field_mut.md#common-mistakes).

## Related constructs

- [`FieldGetter`](./field_getter.md): the supertrait, where wired access is explained in full.
- [`HasFieldMut`](./has_field_mut.md): the consumer side you bound against.
- [`UseField`](../../providers/use_field.md) and [`WithField`](../../providers/with_field.md): the
  provider that implements both halves, and its `WithProvider` alias.
- [`#[cgp_getter]`](../../macros/cgp_getter.md): the macro whose mutable getters take this trait.

The ideas behind it:

- [Consumer and provider traits](/docs/concepts/consumer-and-provider-traits): the split this trait
  is an instance of.

## Source

- [`traits/has_field_mut.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/has_field_mut.rs):
  `MutFieldGetter`, `HasFieldMut`, and the `DerefMut` forwarding
- [`impls/use_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/impls/use_field.rs):
  the `UseField` impls

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
