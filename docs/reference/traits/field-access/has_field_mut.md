---
sidebar_label: 'HasFieldMut'
sidebar_position: 2
---

# `HasFieldMut`

Mutable tag-keyed field access.

## Overview

[`HasField`](./has_field.md) lets an implementation read a field of a context it cannot name, by keying
on the field's name as a type. `HasFieldMut` is the same access with mutation: it hands back a `&mut`
rather than a `&`, so an implementation can change a value in place.

**You do not opt into it.** [`#[derive(HasField)]`](../../derives/derive_has_field.md) always emits
`HasFieldMut` beside `HasField`, so every derived field is mutably accessible and there is no
derive-level way to make one read-only.

## Definition

`HasFieldMut<Tag>` extends [`HasField<Tag>`](./has_field.md) with one method that returns a mutable
borrow:

```rust
pub trait HasFieldMut<Tag>: HasField<Tag> {
    fn get_field_mut(&mut self, tag: PhantomData<Tag>) -> &mut Self::Value;
}
```

It is a **supertrait extension** rather than an alternative: `HasFieldMut<Tag>` requires `HasField<Tag>`,
so the field's type comes from that supertrait's `Value`, and bounding on the mutable form gives you the
read as well. `get_field_mut` takes `&mut self` and returns `&mut Self::Value`, with the
`PhantomData<Tag>` argument naming the field exactly as [`HasField`](./has_field.md)'s `get_field` does.

## Usage

**It is in the prelude**, so `use cgp::prelude::*;` is enough.

The call reads like its immutable sibling, with `PhantomData` naming the field:

```rust
*context.get_field_mut(PhantomData::<Symbol!("counter")>) += 1;
```

An implementation that mutates declares the bound on `Self` the same way it declares a read, and needs
`&mut self` on the method:

```rust
Self: HasFieldMut<Symbol!("counter"), Value = u64>
```

Like its supertrait it carries a diagnostic note pointing at
[`#[derive(HasField)]`](../../derives/derive_has_field.md), so an unsatisfied bound reads as a missing
derive.

## Examples

An implementation that updates a counter on its context:

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
```

**Environmental context, self-targeted** — `App` stands for the application and carries the wiring.

Mutable access passes through a smart pointer too, via a `DerefMut` forwarding impl, so a
`Box<App>` resolves the write to the inner struct.

## When to use it

**Bound on it only when the implementation genuinely writes**, and prefer the read-only bound otherwise.
Requiring mutation where none happens narrows what a caller can pass for no benefit — a context behind a
shared reference satisfies [`HasField`](./has_field.md) and not this.

- **Use [`HasField`](./has_field.md)** whenever the value is only read. This is the overwhelmingly common
  case.
- **Use an [`#[implicit]`](../../attributes/implicit.md) argument** for a read, since it generates the
  immutable bound for you and reads like an ordinary parameter. Implicit arguments cover mutable access
  too, under the access rules on that page, which is the idiomatic route before reaching for this trait
  by hand.
- **Reach for [`MutFieldGetter`](./mut_field_getter.md)** when the mutation must be *wired* rather than
  bounded — the provider-side mirror of this trait.
- **Consider whether the context should be mutated at all.** Much CGP code keeps contexts immutable and
  threads state through handler outputs instead, which composes better with the
  [handler family](../../components/handler/handler.md).

## Under the hood

`HasFieldMut` is implemented for any type whose
[`DerefMut`](https://doc.rust-lang.org/std/ops/trait.DerefMut.html) target implements it, with the target
additionally bounded `'static`. That extra bound is the one asymmetry with
[`HasField`](./has_field.md#under-the-hood)'s `Deref` forwarding, and it is why a borrowed context inside
a smart pointer sometimes resolves the read and not the write.

Like the immutable forwarding, it is marked so the compiler does **not** suggest it in a diagnostic,
which keeps a missing-field error pointed at the struct that lacks the field rather than at the pointer.

The per-field impls themselves come from [`#[derive(HasField)]`](../../derives/derive_has_field.md), one
`HasFieldMut` beside each `HasField`.

## Common Mistakes

**It is not separately opt-in.** The derive always emits it, so there is no way to expose a field as
read-only through the derive. Restricting mutation is a matter of what the implementations declare.

**The `DerefMut` forwarding requires `'static` on the target**, where the `Deref` forwarding does not. A
context holding a borrowed value can therefore satisfy the read and not the write, and the error names
the lifetime rather than the field.

**Bounding on it implies the read.** `Self: HasFieldMut<Tag>` already gives `get_field`, so adding
`HasField<Tag>` alongside is redundant.

**`Value` lives on the supertrait.** Pin it as `HasFieldMut<Symbol!("x"), Value = u64>` — the associated
type is inherited, not redeclared.

**A tuple field is keyed by [`Index<N>`](../../types/index_type.md)**, exactly as for the immutable form.

## Related constructs

- [`HasField`](./has_field.md) — the supertrait, and where the tag-keyed access is explained in full.
- [`MutFieldGetter`](./mut_field_getter.md) — the provider-side mirror of this trait.
- [`#[derive(HasField)]`](../../derives/derive_has_field.md) — generates both impls per field.
- [`#[implicit]`](../../attributes/implicit.md) — the idiomatic way to reach a field, including mutably.
- [`Symbol!`](../../macros/symbol.md) and [`Index`](../../types/index_type.md) — the tags that key a field.
- [`UseField`](../../providers/use_field.md) — the provider that implements the wired form.

The ideas behind it:

- [Impl-side dependencies](/docs/concepts/impl-side-dependencies) — why a field requirement belongs on
  the implementation rather than the interface.

## Source

- [`has_field_mut.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/has_field_mut.rs)
  — `HasFieldMut`, `MutFieldGetter`, and the `DerefMut` forwarding

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
