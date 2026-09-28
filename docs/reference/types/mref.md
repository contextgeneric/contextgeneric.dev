---
title: 'MRef — a borrowed or owned getter value'
sidebar_label: 'MRef'
sidebar_position: 11
description: 'A value that is either a borrow of a T or an owned T, so one getter signature serves both the context that stores a value and the one that computes it.'
---

# `MRef`

A "maybe-reference": a value that is either a borrow of a `T` or an owned `T`, so a getter can return
whichever it has without forcing every implementor to one or the other.

## Overview

`MRef<'a, T>` lets one getter signature serve both the context that already stores a value and the
context that must produce one. Here a **context** is the type the method runs on, which supplies the
values it needs as its fields. A getter that returns `&'a T` forces every context to keep a `T` it
can lend. A getter that returns `T` forces every context to give up ownership, and to clone even
when it could share a reference. `MRef<'a, T>` avoids both constraints by being either case at run
time. A context with the value in a field returns `MRef::Ref` and lends it. A context that computes
or assembles the value returns `MRef::Owned` and transfers it. The caller treats both the same,
because `MRef` derefs to `T`.

CGP's getter machinery uses the type directly, because a getter method's return type decides what
body the macro generates. When a getter returns `MRef<'a, T>` over `&self`, the generated accessor
wraps the borrowed field as `MRef::Ref(...)`. So the common case, reading a stored field, costs
nothing extra, and the same interface still lets a provider elsewhere return an owned value. A
getter can therefore leave open whether the context stores the value or makes it, without splitting
into a separate trait for each case.

Unlike the rest of this section, `MRef` is an ordinary runtime value rather than a type-level
marker. It belongs here because it is a type you name in your own code, as the return type of a
getter.

## Definition

`MRef` is an enum with a borrowed variant and an owned variant, parameterized by a lifetime and an element
type:

```rust
pub enum MRef<'a, T> {
    Ref(&'a T),
    Owned(T),
}
```

`Ref` borrows a `T` for the lifetime `'a`, and `Owned` carries a `T` by value. The lifetime applies
only to the borrowed case, so an `MRef` built from an owned value is effectively unbounded in `'a`.
The enum is an ordinary owned value, with nothing type-level about it, and it is the payload a
getter passes back to its caller. It is in the prelude, so `use cgp::prelude::*;` is enough.

## Behavior

`MRef` behaves like a smart pointer to `T`, which makes the variants interchangeable at the call site. It
implements `Deref<Target = T>` by matching on the variant and returning a `&T` either way, so `&*my_ref`
and any auto-deref method call work regardless of which case is inside. It also implements `AsRef<T>` over
the same logic, giving an explicit `as_ref()` for code that prefers it.

Building an `MRef` takes a single `.into()`, because it implements `From` for both cases. `From<T>`
builds `Owned`, and `From<&'a T>` builds `Ref`, so a value or a reference converts with `.into()`.
When a caller needs ownership unconditionally, `get_or_clone` resolves the enum to a plain `T`. It
returns the owned value as is or clones the borrowed one, and it is available whenever `T: Clone`.
These make up the whole API: the transparent `Deref` and `AsRef`, the `From` impls, and
`get_or_clone`. `MRef` does not implement any other trait, so it is neither `Clone` nor `Debug`.
Code reads a borrowed `MRef` cheaply and promotes it to ownership only on request.

The getter macros recognize `MRef<'_, T>` by its shape: a single-segment path named `MRef` with a
lifetime and a type argument. The same form works for an [`#[implicit]`](../attributes/implicit.md)
argument, which then lends the field as `MRef::Ref`. A differently shaped or fully qualified `MRef`,
such as `cgp::prelude::MRef<'_, T>`, is not recognized. It falls through to the owned form, which
expects a field of that type, and with `'_` in it the definition does not compile.

## Examples

A getter returning `MRef` lets one context lend a stored field and another build the value, and
generic code reads both the same way:

```rust
use cgp::prelude::*;

#[cgp_getter]
pub trait HasGreeting {
    fn greeting(&self) -> MRef<'_, String>;
}

// A provider that builds the value instead of lending a field.
#[cgp_impl(new BuildGreeting)]
impl GreetingGetter {
    fn greeting(&self, #[implicit] name: &str) -> MRef<'_, String> {
        MRef::Owned(format!("Hello, {name}!"))
    }
}

#[derive(HasField)]
pub struct Stored {
    pub greeting: String,
}

#[derive(HasField)]
pub struct Computed {
    pub name: String,
}

delegate_components! {
    Stored {
        GreetingGetterComponent: UseField<Symbol!("greeting")>,
    }
}

delegate_components! {
    Computed {
        GreetingGetterComponent: BuildGreeting,
    }
}

check_components! {
    Stored {
        GreetingGetterComponent,
    }
}

check_components! {
    Computed {
        GreetingGetterComponent,
    }
}

// Generic code reads either case through `Deref`.
pub fn shout<Context: HasGreeting>(context: &Context) -> String {
    context.greeting().to_uppercase()
}

pub fn demo() {
    let stored = Stored {
        greeting: "Hi there".to_owned(),
    };
    let computed = Computed {
        name: "Alice".to_owned(),
    };

    assert_eq!(shout(&stored), "HI THERE");
    assert_eq!(shout(&computed), "HELLO, ALICE!");

    // `UseField` lends the stored field, and `get_or_clone` moves an owned value out.
    assert!(matches!(stored.greeting(), MRef::Ref(_)));
    let owned: String = computed.greeting().get_or_clone();
    assert_eq!(owned, "Hello, Alice!");
}
```

`Stored` and `Computed` are value contexts for a self-targeted getter. `Stored` wires the getter to
[`UseField`](../providers/use_field.md), whose generated accessor wraps the field as `MRef::Ref`.
`Computed` wires it to a provider that formats a new `String` and returns it as `MRef::Owned`.

## When to use it

**Return `MRef<'a, T>` from a getter that some contexts store and others build.** It is the right getter
return type when the value is not always a field the context can lend.

- **Use a plain `&T` return** when every context stores the value and can lend it. `MRef` is useful only
  where some context must produce the value instead.
- **Use an [`#[implicit]`](../attributes/implicit.md) argument** to read a stored field in a
  provider, which is the default for field access. An implicit argument declared as `MRef<'_, T>`
  lends the field as `MRef::Ref`, which suits a body written against a value that may be owned or
  borrowed.
- **Use `MRef` with [`#[cgp_getter]`](../macros/cgp_getter.md)** and the
  [`UseField`](../providers/use_field.md) family, where a getter's return type selects the accessor the
  macro generates.

## Common Mistakes

**`MRef` is not related to [`Life`](life.md).** Its `'a` is an ordinary borrow lifetime on a runtime
value, while `Life<'a>` is a zero-sized type-level lift for provider wiring. They share only the word
"lifetime".

**`get_or_clone` clones only the borrowed case.** It moves an `Owned` value and clones a `Ref` one, so it
is free when the getter already owns the value and costs a clone when it borrowed. Use it only when you
need ownership.

**A qualified path is not recognized.** A getter declared as
`fn greeting(&self) -> cgp::prelude::MRef<'_, String>;` is treated as returning an owned field of
that type, so its `'_` lands in a `HasField` bound and the definition fails with
``error[E0637]: `'_` cannot be used here``. Import `MRef` and write it bare.

**`Deref` makes the variants transparent, so you rarely match on them.** Reading through `&*` or
`as_ref()` works whichever case is inside, and a manual match on `Ref` versus `Owned` usually means the
code should have called `get_or_clone` instead.

## Related constructs

- [`#[cgp_getter]`](../macros/cgp_getter.md): the getter component whose return type may be `MRef`.
- [`#[cgp_auto_getter]`](../macros/cgp_auto_getter.md): the blanket getter, which recognizes the
  same return type.
- [`UseField`](../providers/use_field.md) and [`UseFieldRef`](../providers/use_field_ref.md): the
  providers that wire a getter, and the by-reference variant.
- [`HasField`](../traits/field-access/has_field.md): the field access an `MRef` getter builds on.
- [`#[implicit]`](../attributes/implicit.md): the default way to read a field, which also accepts an
  `MRef` type.
- [`Life`](life.md): a different, type-level use of a lifetime, not to be confused with this one.

The ideas behind it:

- [Implicit arguments](/docs/concepts/implicit-arguments): reading a value out of a context, which is
  what an `MRef` getter returns.

## Source

- The type, with its `Deref`, `AsRef`, the two `From` impls, and `get_or_clone`:
  [`mref.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/types/mref.rs)
- The macro logic that recognizes an `MRef<'a, T>` getter return type:
  [`field/parse.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/functions/field/parse.rs),
  with the `MRef::Ref(...)` body emitted by
  [`getter/field_mode.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/getter/field_mode.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
