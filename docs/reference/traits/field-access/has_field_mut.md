---
title: 'HasFieldMut — mutable field access by tag'
sidebar_label: 'HasFieldMut'
sidebar_position: 2
description: 'Mutable tag-keyed field access, letting an implementation change a field of a context it cannot name, as the mutable counterpart of HasField.'
---

# `HasFieldMut`

Mutable tag-keyed field access.

## Overview

[`HasField`](./has_field.md) lets an implementation read a field of a
[**context**](/docs/reference/glossary#context), the type the method runs on, which supplies the
values it needs as its fields, by keying on the field's name as a type. `HasFieldMut` is the same
access with mutation: it hands back a `&mut` rather than a `&`, so an implementation can change a
value in place.

**You do not opt into it.** [`#[derive(HasField)]`](../../derives/derive_has_field.md) always emits
`HasFieldMut` beside `HasField`, so every derived field is mutably accessible, and there is no
derive-level way to make one read-only.

## Definition

`HasFieldMut<Tag>` extends [`HasField<Tag>`](./has_field.md) with one method that returns a mutable
borrow:

```rust
pub trait HasFieldMut<Tag>: HasField<Tag> {
    fn get_field_mut(&mut self, tag: PhantomData<Tag>) -> &mut Self::Value;
}
```

It is a [supertrait](/docs/reference/glossary#supertrait) extension rather than an alternative:
`HasFieldMut<Tag>` requires `HasField<Tag>`, so the field's type comes from that supertrait's
`Value`, and bounding on the mutable form gives the read as well. `get_field_mut` takes `&mut self`
and names the field with `PhantomData<Tag>`, as `get_field` does.

## Usage

It is in the prelude, so `use cgp::prelude::*;` is enough. An implementation that mutates declares
the bound on `Self`, pinning `Value` through the supertrait, and takes `&mut self`:

```rust
Self: HasFieldMut<Symbol!("counter"), Value = u64>
```

The call reads like its immutable sibling:

```rust
*self.get_field_mut(PhantomData::<Symbol!("counter")>) += 1;
```

## Examples

A provider that increments a counter on its context, and a generic function that resets it through
a `Box`:

```rust
use cgp::prelude::*;

#[cgp_component(Counter)]
pub trait CanCount {
    fn count(&mut self);
}

#[cgp_impl(new IncrementCounter)]
impl Counter
where
    Self: HasFieldMut<Symbol!("counter"), Value = u64>,
{
    fn count(&mut self) {
        *self.get_field_mut(PhantomData::<Symbol!("counter")>) += 1;
    }
}

#[derive(HasField)]
pub struct App {
    pub counter: u64,
}

delegate_components! {
    App {
        CounterComponent: IncrementCounter,
    }
}

check_components! {
    App {
        CounterComponent,
    }
}

pub fn reset<Context>(context: &mut Context)
where
    Context: HasFieldMut<Symbol!("counter"), Value = u64>,
{
    *context.get_field_mut(PhantomData) = 0;
}

pub fn demo() {
    let mut app = App { counter: 0 };
    app.count();
    app.count();
    assert_eq!(app.counter, 2);

    // `Box<App>` has the field mutably through the `DerefMut` forwarding impl.
    let mut boxed = Box::new(App { counter: 5 });
    reset(&mut boxed);
    assert_eq!(boxed.counter, 0);
}
```

`App` is an [environmental context](/docs/reference/glossary#environmental-context) that carries the
counter, and the component is [self-targeted](/docs/reference/glossary#self-targeted-component).

## When to use it

**Bound on it only when the implementation writes**, and use [`HasField`](./has_field.md) otherwise.
Requiring mutation where none happens narrows what a caller can pass, since a context behind a
shared reference satisfies `HasField` but not this.

- **An [`#[implicit]`](../../attributes/implicit.md) argument** covers mutable access too, under the
  access rules on that page, and is the idiomatic route before a hand-written bound.
- **[`MutFieldGetter`](./mut_field_getter.md)** is the provider-side form, for a getter whose
  mutable access a context chooses by wiring.
- **Consider whether the context should be mutated at all.** Much CGP code keeps contexts immutable
  and passes state through handler outputs instead, which composes with the
  [handler family](../../components/handler/handler.md).

## Under the hood

`#[derive(HasField)]` emits one `HasFieldMut` impl beside each `HasField` impl. The trait module
adds one forwarding impl through
[`DerefMut`](https://doc.rust-lang.org/std/ops/trait.DerefMut.html):

```rust
#[diagnostic::do_not_recommend]
impl<Context, Tag, Target, Value> HasFieldMut<Tag> for Context
where
    Context: DerefMut<Target = Target>,
    Target: HasFieldMut<Tag, Value = Value> + 'static,
{
    fn get_field_mut(&mut self, tag: PhantomData<Tag>) -> &mut Self::Value {
        self.deref_mut().get_field_mut(tag)
    }
}
```

The `'static` bound on the target is the one asymmetry with [`HasField`](./has_field.md)'s `Deref`
forwarding, which passes the borrow through a helper instead. So a smart pointer to a type holding a
borrowed value forwards the read and not the write.

## Common Mistakes

**It is not separately opt-in.** The derive always emits it, so a field cannot be exposed read-only
through the derive; restricting mutation is a matter of what implementations declare.

**The `DerefMut` forwarding requires `'static` on the target**, where the `Deref` forwarding does
not. A context behind a smart pointer to borrowed data satisfies the read and not the write, and the
error names the lifetime rather than the field.

**Bounding on it implies the read.** `Self: HasFieldMut<Tag>` already gives `get_field`, so adding
`HasField<Tag>` beside it is redundant.

## Related constructs

- [`HasField`](./has_field.md): the supertrait, where tag-keyed access is explained in full.
- [`MutFieldGetter`](./mut_field_getter.md): the provider-side form of this trait.
- [`#[derive(HasField)]`](../../derives/derive_has_field.md): generates both impls per field.
- [`#[implicit]`](../../attributes/implicit.md): the idiomatic way to reach a field, including
  mutably.

The ideas behind it:

- [Impl-side dependencies](/docs/concepts/impl-side-dependencies): why a field requirement belongs
  on the implementation rather than the interface.

## Source

- [`traits/has_field_mut.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/has_field_mut.rs):
  `HasFieldMut`, `MutFieldGetter`, and the `DerefMut` forwarding

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
