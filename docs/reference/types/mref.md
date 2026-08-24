---
sidebar_label: 'MRef'
sidebar_position: 6
---

# `MRef`

A "maybe-reference": a value that is either a borrow of a `T` or an owned `T`, so a getter can return
whichever it has without forcing every implementor to one or the other.

## Overview

`MRef<'a, T>` lets one getter signature serve both the context that already stores a value and the
context that must produce one. Here a **context** is the type the capability runs against, which supplies
the values it needs as its own fields. A getter that returns `&'a T` forces every context to keep a `T`
it can lend; a getter that returns `T` forces every context to hand over ownership, cloning even when it
has a perfectly good reference to share. `MRef<'a, T>` removes that dilemma by being either case at run
time: a context with the value in a field returns `MRef::Ref` and lends it, while a context that computes
or assembles the value returns `MRef::Owned` and gives it away. The caller treats both the same, because
`MRef` derefs to `T`.

The type earns its place in CGP's getter machinery, where a getter method's return type decides what body
the macro generates. When a getter returns `MRef<'a, T>` over `&self`, the generated accessor wraps the
borrowed field as `MRef::Ref(...)`, so the common case, reading a stored field, costs nothing extra,
while the same interface still lets a provider elsewhere return an owned value. This lets a
getter abstract over "do I have this value, or do I make it?" without splitting into two traits.

Unlike the rest of this section, `MRef` is an ordinary runtime value rather than a type-level marker.
It is here because it is the one type in the group you write on purpose, as the return type of a getter.

## Definition

`MRef` is a two-variant enum parameterized by a lifetime and an element type:

```rust
pub enum MRef<'a, T> {
    Ref(&'a T),
    Owned(T),
}
```

`Ref` borrows a `T` for the lifetime `'a`; `Owned` carries a `T` by value. The lifetime applies only to
the borrowed case, so an `MRef` built from an owned value is effectively unbounded in `'a`. It is an
ordinary owned value, with nothing type-level about it, and it is the payload a getter passes back to its
caller.

## Behavior

`MRef` behaves like a smart pointer to `T`, which makes the two variants interchangeable at the
call site. It implements `Deref<Target = T>` by matching on the variant and returning a `&T` either way,
so `&*my_ref` and any auto-deref method call work regardless of which case is inside. It also implements
`AsRef<T>` over the same logic, giving an explicit `as_ref()` for code that prefers it.

Building an `MRef` is frictionless, because it implements `From` in both directions: `From<T>` builds
`Owned` and `From<&'a T>` builds `Ref`, so a value or a reference converts with `.into()`. When a caller
needs ownership unconditionally, `get_or_clone` resolves the enum to a plain `T`, returning the owned
value as is or cloning the borrowed one, and is available whenever `T: Clone`. These three pieces, the
transparent `Deref` and `AsRef`, the two `From` impls, and `get_or_clone`, are the whole surface: a
borrowed `MRef` is read cheaply and promoted to ownership only when asked.

## Examples

`MRef` is the return type of a getter that should work whether the context stores the value or produces
it. A borrowed field and a freshly built value have the same type and are read the same way:

```rust
use cgp::prelude::*;

let stored = String::from("hello");

// a context lending a stored value:
let borrowed: MRef<'_, String> = MRef::from(&stored);
assert_eq!(&*borrowed, "hello");

// a provider returning a freshly built value through the same type:
let made: MRef<'_, String> = MRef::from(String::from("world"));
assert_eq!(made.as_ref(), "world");

// promote either to an owned value when ownership is required:
let owned: String = borrowed.get_or_clone();
assert_eq!(owned, "hello");
```

Both `borrowed` and `made` have the same type and are consumed the same way; only the construction
differs, and `get_or_clone` clones the borrowed case while moving the owned one.

## When to use it

**Return `MRef<'a, T>` from a getter that some contexts store and others build.** It is the getter return
mode to reach for when the value is not always a field the context can lend.

- **Use a plain `&T` return** when every context stores the value and can lend it. `MRef` earns its keep
  only where some context must produce the value instead.
- **Use an [`#[implicit]`](../attributes/implicit.md) argument** to read a stored field in a provider,
  which is the default for field access; an implicit argument can itself be typed `MRef<'_, T>` when the
  field may be lent or produced.
- **Reach for `MRef` with [`#[cgp_getter]`](../macros/cgp_getter.md)** and the
  [`UseField`](../providers/use_field.md) family, where a getter's return type selects the accessor the
  macro generates.

## Common Mistakes

**`MRef` is not related to [`Life`](life.md).** Its `'a` is an ordinary borrow lifetime on a runtime
value; `Life<'a>` is a zero-sized type-level lift for provider wiring. The shared word "lifetime" is the
only thing they have in common.

**`get_or_clone` clones only the borrowed case.** It moves an `Owned` value and clones a `Ref` one, so it
is free when the getter already owns the value and costs a clone when it borrowed. Reach for it only when
you genuinely need ownership.

**`Deref` makes the variants transparent, so you rarely match on them.** Reading through `&*` or
`as_ref()` works whichever case is inside, and matching on `Ref` versus `Owned` by hand is usually a sign
the value should have been promoted with `get_or_clone` instead.

## Related constructs

- [`#[cgp_getter]`](../macros/cgp_getter.md) — the getter component whose return type may be `MRef`.
- [`UseField`](../providers/use_field.md) and [`UseFieldRef`](../providers/use_field_ref.md) — the
  providers that wire a getter, and the by-reference variant.
- [`HasField`](../traits/field-access/has_field.md) — the field access an `MRef` getter builds on.
- [`#[implicit]`](../attributes/implicit.md) — the default way to read a field, which also accepts an
  `MRef` type.
- [`Life`](life.md) — a different, type-level use of a lifetime, not to be confused with this one.

The ideas behind it:

- [Implicit arguments](/docs/concepts/implicit-arguments) — reading a value out of a context, which is
  what an `MRef` getter returns.

## Source

- The type, with its `Deref`, `AsRef`, the two `From` impls, and `get_or_clone`:
  [`mref.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/types/mref.rs)
- The macro logic that recognizes an `MRef<'a, T>` getter return type:
  [`field/parse.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/functions/field/parse.rs),
  with the `MRef::Ref(...)` body emitted by
  [`getter/field_mode.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/getter/field_mode.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
