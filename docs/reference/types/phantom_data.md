---
sidebar_label: 'PhantomData'
sidebar_position: 1
---

# `PhantomData`

The standard-library marker that lets a type carry a type parameter without storing a value of it.
It makes CGP's zero-sized providers and type-level markers legal, and it is the value you pass to
give a function a type instead of data.

## Overview

`PhantomData` is a standard-library type, not a CGP type. It comes from
[`core::marker`](https://doc.rust-lang.org/std/marker/struct.PhantomData.html). It has a page here because
it appears throughout CGP code and does one specific job. A reader who has only seen it as noise in a
struct definition will misread much of what CGP generates.

CGP uses it for **declaring** a type that is generic over something it never stores, and for **passing** a
type to a function as a zero-sized value. The sections below work through both uses.

## Why a marker needs it

Rust requires a type to *use* every generic parameter it declares, in a field or in a bound. A type-level
marker conflicts with that rule by design. It is generic over a parameter without storing a value of it,
because the parameter is part of the type's identity rather than its data.

A provider is the clearest case. `Multiply<Field>` stores nothing at run time, yet it must be generic over
`Field` so that the wiring can tell one instance from another. Written as a plain unit struct, it does not
compile:

```rust
pub struct Multiply<Field>;
// error[E0392]: type parameter `Field` is never used
```

`PhantomData<Field>` is the fix. It is a zero-sized field that tells the compiler to treat the type as
though it holds a `Field`, so the parameter counts as used and the struct stays empty at run time:

```rust
use core::marker::PhantomData;

pub struct Multiply<Field>(pub PhantomData<Field>);
```

`Multiply<u32>` and `Multiply<u64>` are now distinct types the wiring can choose between, and the struct
is still zero-sized.

## Definition

`PhantomData` is a zero-sized unit-like struct with one type parameter:

```rust
pub struct PhantomData<T: ?Sized>;
```

That definition has the shape the previous section said the compiler rejects: a struct generic over a `T`
that it stores nowhere. `PhantomData` compiles anyway because the compiler knows it. It carries a
`#[lang = "phantom_data"]` attribute that the compiler recognizes, so it alone is exempt from the
unused-parameter rule, and every other marker type can use it. Those types satisfy the rule by *holding* a
`PhantomData` field rather than by being one.

The parameter is `?Sized`, so `T` may be any type, sized or not. A value of `PhantomData<T>` occupies no
space and holds nothing. Its only content is the type `T`, recorded in its own type. You construct one by
writing `PhantomData`. Where inference cannot supply the type, you name it with a turbofish, as in
`PhantomData::<u32>`.

`PhantomData<T>` also tells the compiler how the surrounding type relates to `T`, beyond satisfying the
used-parameter rule. It sets **variance** (whether a longer lifetime may stand in for a shorter one), drop
checking, and the auto traits such as `Send` and `Sync`. Most CGP markers do not depend on which
relationship they get, so they use the plain `PhantomData<T>`. [`Life`](life.md) is the exception. It
wraps its lifetime as `PhantomData<*mut &'a ()>` to force invariance, so that the compiler treats two
lifetime instantiations as different types. That page explains why.

## How CGP uses it

The declaring side and the passing side look different in the code.

**On the declaring side, `PhantomData` fills the field of a zero-sized type**, the way `Multiply<Field>`
above carries it. The same pattern runs through CGP's recursive type-level lists: a
[`Chars`](chars.md) node holds its next node in a `PhantomData<Tail>`, a
[`PathCons`](path_cons.md) holds both its head and tail that way, and a [`Field`](field.md) holds
its type-level name tag in a `PhantomData<Tag>` beside the one real value it stores. In each case the
parameter is part of the type's identity, and the type does not keep a value of it at run time.

**On the passing side, `PhantomData::<T>` is a zero-sized value that passes the type `T` to a
function.** A method that must know which field, which computation, or which tag is meant takes a
`PhantomData<T>` argument, and the caller supplies the type through it:

```rust
// which field to read:
let name = self.get_field(PhantomData::<Symbol!("name")>);

// which computation to run:
let output = context.compute(PhantomData::<Doubled>, input);
```

Here the **context** is the type the capability runs against. It supplies the values the capability
needs as its own fields. The `PhantomData` argument carries nothing. It exists so that type
inference selects the right [`HasField`](../traits/field-access/has_field.md) impl or the right
handler. In effect, the call passes a type where a value is expected, and `PhantomData::<T>` is the
empty value that stands for the type `T`.

**You rarely write either role yourself, because a higher-level construct produces each for you.**

- On the declaring side, the `new` keyword of [`#[cgp_impl]`](../macros/cgp_impl.md) declares the
  provider struct: a `PhantomData` field per generic parameter, or a bare `struct Foo;` when there are
  none.
- On the passing side, [`#[implicit]`](../attributes/implicit.md) and
  [`#[cgp_auto_getter]`](../macros/cgp_auto_getter.md) read a context field without the tag, so your code
  does not contain a `get_field(PhantomData::<...>)` call.

You meet `PhantomData` directly only when you work below these constructs.

## Examples

At a call site, `PhantomData::<Tag>` selects which field a getter reads:

```rust
use cgp::prelude::*;

#[cgp_impl(new GreetHello)]
impl Greeter
where
    Self: HasField<Symbol!("name"), Value = String>,
{
    fn greet(&self) {
        let name = self.get_field(PhantomData::<Symbol!("name")>);
        println!("Hello, {name}!");
    }
}
```

You meet this form in an expansion rather than write it. The idiomatic version uses an
[`#[implicit]`](../attributes/implicit.md) argument and does not show `PhantomData` at all.

A type with several parameters uses one `PhantomData` over a tuple:

```rust
pub struct Add<Left, Right>(pub PhantomData<(Left, Right)>);
```

## When to use it

**You write `PhantomData` yourself only to declare a marker field or to pass a type at a call site.**
Everywhere else the macros insert it for you.

- **Declare a `PhantomData` field** when you write a zero-sized struct that is generic over a type without
  storing a value of it, such as a provider struct you define yourself rather than through
  [`#[cgp_new_provider]`](../macros/cgp_provider.md). One `PhantomData` over a tuple covers several
  parameters.
- **Pass `PhantomData::<T>`** at a call site that takes a type-level selector, such as `get_field` or a
  handler's `compute`. This is the common case you write yourself, and the turbofish names the type when
  inference cannot.
- **Use [`Life`](life.md), not a bare `PhantomData`, for a lifetime** that must stay invariant. A plain
  `PhantomData<&'a ()>` is covariant, which is the wrong relationship for a dependency marker.

## Common Mistakes

**It costs nothing at run time.** `PhantomData<T>` is zero-sized whatever `T` is, so a struct full of
`PhantomData` fields is still zero-sized, and a `PhantomData` argument compiles to nothing. A
`PhantomData` in a signature does not mean a hidden allocation.

**`PhantomData` and `PhantomData::<T>` differ only by whether the type is inferred.** In a struct field
the type comes from the field's declared type, so plain `PhantomData` is enough. At a call site where
nothing fixes `T`, the turbofish `PhantomData::<T>` states it, and omitting it gives a
"type annotations needed" error rather than a wrong result.

**A bare `PhantomData<&'a ()>` is covariant.** If you write a lifetime marker yourself and let a longer
lifetime stand in for a shorter one, the compiler may pick a provider wired for a different lifetime.
Use [`Life<'a>`](life.md), whose `PhantomData<*mut &'a ()>` forces invariance, when the lifetime is an
exact identity.

**A generic marker needs its `PhantomData` field.** A struct that is generic over a `T` without storing a
value of it fails to compile with `error[E0392]`.

## Related constructs

- [`Field`](field.md): holds its type-level name tag in a `PhantomData<Tag>` beside its value.
- [`Life`](life.md): a lifetime lifted into a type through a deliberately invariant `PhantomData`.
- [`Chars`](chars.md) and [`PathCons`](path_cons.md): recursive type-level lists whose tails are
  `PhantomData` markers.
- [`Symbol!`](../macros/symbol.md) and [`Index`](index_type.md): the tags a `PhantomData::<Tag>`
  argument usually carries.
- [`HasField`](../traits/field-access/has_field.md): whose `get_field` takes the tag as a
  `PhantomData` argument.
- [`#[cgp_new_provider]`](../macros/cgp_provider.md): declares a provider struct's `PhantomData` field
  for you.

The ideas behind it:

- [Consumer and provider traits](/docs/concepts/consumer-and-provider-traits): where the zero-sized
  provider structs that carry `PhantomData` come from.
- [Type-level DSLs](/docs/concepts/type-level-dsls): encoding a program as types, which is why a type
  gets passed as a value at all.

## Source

- The type is standard-library:
  [`core::marker::PhantomData`](https://doc.rust-lang.org/std/marker/struct.PhantomData.html), whose
  documentation covers the variance, drop-check, and auto-trait effects in full.
- CGP re-exports it through the prelude, so `use cgp::prelude::*;` brings `PhantomData` into scope
  alongside the rest.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
