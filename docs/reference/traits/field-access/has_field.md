---
sidebar_label: 'HasField'
sidebar_position: 1
---

# `HasField`

Reading a field by a type-level name, from a context you cannot name.

## Overview

An implementation written against a **context** — the type the capability runs against, which supplies
the values it needs as its fields — is generic over that context and cannot name its concrete type. So it
cannot write `self.name`. Yet reading a value out of the context is the commonest thing such an
implementation does.

`HasField<Tag>` closes the gap. The field's *name* becomes a type, and the implementation asks for it as
an ordinary trait bound:

```rust
Self: HasField<Symbol!("name"), Value = String>
```

Any context with a matching field satisfies that bound. Nothing is compared by string at run time —
[`Symbol!("name")`](../../macros/symbol.md) is a type, so the compiler resolves which field is meant and the
read compiles to a direct field access.

This is the foundation the whole ergonomic surface stands on. An
[`#[implicit]`](../../attributes/implicit.md) argument, a [`#[cgp_auto_getter]`](../../macros/cgp_auto_getter.md)
method, and a [`UseField`](../../providers/use_field.md) wiring entry all generate this bound from a name you
already wrote. **You will read `HasField` far more often than you write it**, and the impls come from
[`#[derive(HasField)]`](../../derives/derive_has_field.md).

## Definition

`HasField<Tag>` carries the field's type as an associated `Value` and returns a reference to it:

```rust
pub trait HasField<Tag> {
    type Value;

    fn get_field(&self, _tag: PhantomData<Tag>) -> &Self::Value;
}
```

`Tag` is a type-level name: [`Symbol!("field_name")`](../../macros/symbol.md) for a named field,
[`Index<N>`](../../types/index.md) for a tuple field. `Value` is the field's type, exposed as an associated
type so a bound can pin it or leave it open — `HasField<Symbol!("name")>` accepts a field of any type,
while `HasField<Symbol!("name"), Value = String>` requires a `String`. `get_field` takes `&self` and
returns `&Self::Value`, a borrow of the field; its `PhantomData<Tag>` argument carries no data and only
lets a call site say *which* field it means when several `HasField` impls are in scope, which is why a
read is written `self.get_field(PhantomData::<Symbol!("name")>)`. The trait also carries a
`#[diagnostic::on_unimplemented]` note pointing at
[`#[derive(HasField)]`](../../derives/derive_has_field.md), so a missing field reads as a missing derive
rather than an opaque trait failure.

## Usage

**It is in the prelude**, so `use cgp::prelude::*;` is enough.

**Four neighbours complete the picture**, each on its own page: [`HasFieldMut`](./has_field_mut.md) for
mutable access, [`FieldGetter`](./field_getter.md) for the provider-side mirror that gets wired, and
[`MapField`](./map_field.md) with [`FieldMapper`](./field_mapper.md) for the lifetime-safe form nested
accessors use.

## Examples

An implementation states the field it needs and reads it:

```rust
use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

#[cgp_impl(new GreetHello)]
impl Greeter
where
    Self: HasField<Symbol!("name"), Value = String>,
{
    fn greet(&self) {
        println!("Hello, {}!", self.get_field(PhantomData::<Symbol!("name")>));
    }
}

#[derive(HasField)]
pub struct Person {
    pub name: String,
}

delegate_components! {
    Person {
        GreeterComponent: GreetHello,
    }
}
```

`Person` derives the access, so it satisfies exactly the bound `GreetHello` requires and the wiring
compiles. **Value context, self-targeted** — the wired type is the data the capability runs against.

**Written idiomatically, none of that bound is visible.** The same read is an
[`#[implicit]`](../../attributes/implicit.md) argument:

```rust
#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self, #[implicit] name: &str) {
        println!("Hello, {name}!");
    }
}
```

which generates the identical bound and the identical read. The explicit form above appears in an error
message or an expansion, not in code you type.

Field access also passes through smart pointers with no extra work, because the trait carries a `Deref`
forwarding impl:

```rust
let boxed: Box<Person> = Box::new(Person { name: "Alice".to_owned() });

assert_eq!(boxed.get_field(PhantomData::<Symbol!("name")>), "Alice");
```

## When to use it

**Bound against `HasField` only when the ergonomic constructs cannot do the job**, which is rarely. The
ordering is settled and worth following.

- **Use an [`#[implicit]`](../../attributes/implicit.md) argument by default.** It reads a field of the
  implementation's own context as a plain parameter, generating this bound for you. It covers the common
  case, including a field several implementations each read.
- **Use [`#[cgp_auto_getter]`](../../macros/cgp_auto_getter.md)** when the read must be a *named* capability,
  when the field lives on a type other than the context, or when the getter should carry a type inferred
  from the field — the three cases an implicit argument cannot reach.
- **Use [`#[cgp_getter]`](../../macros/cgp_getter.md) with [`UseField`](../../providers/use_field.md)** only
  when a context needs to choose *which* field the getter reads. That is the advanced case and costs a
  wiring line per context.
- **Write the bound by hand** when none of those fit — a bound on a type that is not `Self`, say. It is an
  ordinary trait bound.

Three neighbours are easy to confuse with it. [`HasFields`](../shape/has_fields.md) — plural — is the
whole-shape view, for code that must process *every* field rather than one named one; the two are
complementary and often derived together. [`HasFieldMut`](./has_field_mut.md) is the same access with
mutation, not an alternative. And [`FieldGetter`](./field_getter.md) is not an alternative either but the
provider-side mirror: you bound against `HasField` and wire `FieldGetter`.

## Under the hood

The per-field impls come almost entirely from
[`#[derive(HasField)]`](../../derives/derive_has_field.md). What the trait module itself supplies is the
blanket impls that let the access compose, and two of them belong to this trait.

**Smart-pointer forwarding.** `HasField` is implemented for any type whose
[`Deref`](https://doc.rust-lang.org/std/ops/trait.Deref.html) target implements it, so a `Box<Person>` or
a newtype resolves a read to the inner struct. It is marked so the compiler does **not** suggest it in a
diagnostic, which keeps a missing-field error pointed at the struct that lacks the field rather
than at the pointer.

**`UseContext` as a field getter.** The provider side connects to wiring through one impl:
[`UseContext`](../../providers/use_context.md) implements [`FieldGetter`](./field_getter.md) for any context
that already has the field, delegating straight through.

```rust
impl<Context, Tag, Field> FieldGetter<Context, Tag> for UseContext
where
    Context: HasField<Tag, Value = Field>,
{
    type Value = Field;

    fn get_field(context: &Context, _tag: PhantomData<Tag>) -> &Self::Value {
        context.get_field(PhantomData)
    }
}
```

The remaining blanket impls belong to the neighbours: [`HasFieldMut`](./has_field_mut.md) carries the
`DerefMut` forwarding, and [`MapField`](./map_field.md) is free for every `HasField` whose tag is
`'static`.

## Common Mistakes

**Two spellings of a name are unrelated types.** `Symbol!("first_name")` and `Symbol!("firstName")` have
nothing to do with each other, and a mismatch reports as a missing `HasField` bound rather than as a
typo. This is the usual cause of a read that "should" work.

**A tuple field is keyed by [`Index<N>`](../../types/index.md), never by a `Symbol!` of the number.**
`Index<0>` and `Symbol!("0")` are different types.

**`Value` can be pinned or left open.** Omitting `Value = T` when you meant to pin it produces an
inference failure further along rather than at the bound.

**The `PhantomData` argument is required and carries the tag.** `self.get_field(PhantomData)` works only
where inference can determine the tag from context; at an ambiguous site write
`PhantomData::<Symbol!("name")>`.

**It returns a reference, always.** There is no owning read; cloning is the caller's business, and an
[`#[implicit]`](../../attributes/implicit.md) argument inserts a `.clone()` for you when the
parameter is owned.

## Related constructs

- [`#[derive(HasField)]`](../../derives/derive_has_field.md) — generates the per-field impls; what a context
  writes.
- [`HasFieldMut`](./has_field_mut.md) — the mutable extension.
- [`FieldGetter`](./field_getter.md) — the provider-side mirror that gets wired.
- [`MapField`](./map_field.md) — the lifetime-safe form for reaching into a nested value.
- [`HasFields`](../shape/has_fields.md) — the plural, whole-shape counterpart.
- [`Symbol!`](../../macros/symbol.md) and [`Index`](../../types/index.md) — the tags that key a field.
- [`#[implicit]`](../../attributes/implicit.md) — the idiomatic way to read a field.
- [`#[cgp_auto_getter]`](../../macros/cgp_auto_getter.md) and [`#[cgp_getter]`](../../macros/cgp_getter.md) —
  getter traits over the same access.
- [`UseField`](../../providers/use_field.md) — the provider-side implementation of `FieldGetter`.
- [`HasBuilder`](../builder/has_builder.md) — where `HasField` reappears on a partial record, gated on presence.

The ideas behind it:

- [Impl-side dependencies](/docs/concepts/impl-side-dependencies) — why a field requirement belongs on
  the implementation rather than the interface.
- [Implicit arguments](/docs/concepts/implicit-arguments) — the ergonomic surface built on this trait.

## Source

- [`has_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/has_field.rs)
  — `HasField`, `FieldGetter`, the `Deref` forwarding, and the `UseContext` impl

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
