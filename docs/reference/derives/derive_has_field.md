---
sidebar_label: '#[derive(HasField)]'
---

# `#[derive(HasField)]`

Per-field accessors keyed by a type-level tag.

## Overview

An implementation written against a **context** — the type the capability runs against, which supplies the
values it needs as its fields — cannot name that context's concrete type. It is generic over it, so
`self.name` is not something it can write. Yet reading a field out of the context is the most common thing
such an implementation does.

`#[derive(HasField)]` is what closes that gap. It gives each of a struct's fields a *type-level name*, so an
implementation can ask for "a `String` field called `name`" as an ordinary trait bound and receive the field
without knowing what type it came from:

```rust
Self: HasField<Symbol!("name"), Value = String>
```

Any struct that derives `HasField` and happens to have such a field satisfies that bound. Nothing is matched
by string at runtime — [`Symbol!("name")`](../macros/symbol.md) is a type, so the compiler resolves the
lookup during compilation and the read compiles down to a direct field access.

**You will derive this constantly and almost never name `HasField` by hand.** It is the foundation the
ergonomic surface stands on: an [`#[implicit]`](../attributes/implicit.md) argument, a
[`#[cgp_auto_getter]`](../macros/cgp_auto_getter.md) method, and a [`UseField`](../providers/use_field.md)
wiring entry all generate the bound above from a name you already wrote. What you write is the derive on the
struct.

## Using it

The macro is a plain derive on a struct. It takes no arguments and has no helper attributes:

```rust
#[derive(HasField)]
pub struct Person {
    pub name: String,
    pub age: u8,
}
```

It accepts any struct shape. What differs between them is only how each field's tag is computed.

### Named fields

A named field is keyed by [`Symbol!`](../macros/symbol.md), the type-level string of its identifier. The
struct above gains access keyed by `Symbol!("name")` and `Symbol!("age")`.

A field written as a [raw identifier](https://doc.rust-lang.org/reference/identifiers.html) is keyed by its
*logical* name, with the `r#` stripped — a field `r#type` is keyed by `Symbol!("type")`, not
`Symbol!("r#type")`. The generated accessor still borrows the real field.

### Tuple fields

A tuple field has no name, so it is keyed by [`Index<N>`](../types/index.md), the type-level number of its
position:

```rust
#[derive(HasField)]
pub struct Rectangle(pub f64, pub f64);
```

`Rectangle` gains access keyed by `Index<0>` and `Index<1>`. A tuple field is never keyed by a `Symbol!` of
its position — `Index<0>` and `Symbol!("0")` are different types.

### Unit structs

A unit struct has no fields, so there is nothing to key. The derive accepts it and emits nothing at all
rather than reporting an error:

```rust
#[derive(HasField)]
pub struct App;
```

This is worth knowing because it is silent. Deriving `HasField` on a fieldless context is harmless and does
nothing, which is the right outcome — such a context carries choices rather than data — but it also means a
mistake that leaves a struct fieldless produces no complaint here, only an unsatisfied bound later.

### Generic structs

Generic parameters, lifetimes, and a `where` clause are all carried onto the generated access, so a generic
struct works with no extra ceremony:

```rust
#[derive(HasField)]
pub struct Wrapper<T> {
    pub value: T,
}
```

`Wrapper<T>` gains access keyed by `Symbol!("value")` whose value type is `T`, for every `T`.

### What it does not accept

The derive parses its input as a struct, so applying it to an enum or a union fails at parse time. The
whole-shape view of a struct *or* an enum is a different derive,
[`#[derive(HasFields)]`](./derive_has_fields.md) — note the plural — and the two are frequently derived
together.

## Examples

An implementation states the field it needs as a bound, and the derive is what lets a concrete context
satisfy it. First the implementation, asking for a `String` field called `name`:

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
```

Then a context that derives the access and chooses that implementation:

```rust
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

Because `Person` derives `HasField`, it satisfies exactly the bound `GreetHello` requires, so
`person.greet()` compiles and prints the person's name. Remove the `name` field and the wiring stops
compiling — the requirement is checked, not assumed.

**Written idiomatically, none of that bound is visible.** The same read is an
[`#[implicit]`](../attributes/implicit.md) argument, which looks like an ordinary function parameter:

```rust
#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self, #[implicit] name: &str) {
        println!("Hello, {name}!");
    }
}
```

That version generates the same `HasField` bound and the same field read. The explicit form above is what
you *read* — in generated code, in a compiler error — rather than what you write.

Field access also passes through smart pointers with no extra derive. `Box<Person>` and any newtype that
dereferences to `Person` resolve a field read to the inner struct, so wrapping a context does not break the
implementations that read from it.

## When to reach for it, and when not

**Derive it on every context whose fields an implementation reads.** There is little judgement here: the
derive is how a struct's fields become visible to the trait system, it costs one line, and every construct
that reads a field needs it. A context that derives nothing can still be wired and can still implement
consumer traits — it just cannot supply values.

What *is* a judgement call is how you read the field once the derive is in place, and the ordering is
settled.

- **Use an [`#[implicit]`](../attributes/implicit.md) argument by default.** It reads a field of the
  implementation's own context as a plain parameter, with no trait to declare. This covers the common case,
  including a field that several implementations each read.
- **Use [`#[cgp_auto_getter]`](../macros/cgp_auto_getter.md) when the read needs to be a named
  capability**, when the field lives on a type other than the context, or when the getter should carry a
  type inferred from the field — the three cases an implicit argument cannot reach.
- **Write the `HasField` bound by hand only when neither fits**, which is rare. When you do, remember it is
  an ordinary trait bound and can carry `Value = T` to pin the field's type.
- **Reach for [`#[derive(HasFields)]`](./derive_has_fields.md) instead when code must process the whole
  shape** rather than one named field — serializing a struct, folding over its fields, building it
  generically. The two answer different questions and are often derived together.

One thing this derive is not: it is **not** what makes a struct an extensible record. It gives indexed
access to fields that already exist. Building a struct up field by field is
[`#[derive(BuildField)]`](./derive_build_field.md), and the umbrella that includes both is
[`#[derive(CgpData)]`](./derive_cgp_data.md).

## Under the hood

:::note

### Advanced

This section shows what the derive generates. You do not need it to use `#[derive(HasField)]`, but a missing
field is reported against the generated bound with the tag fully expanded, so reading one expansion makes
that error legible. `cargo cgp expand` prints the same thing for your own code, with the tags resugared.

:::

The derive leaves the struct definition untouched and adds **two impls per field** — a read accessor and a
mutable one. From this input:

```rust
#[derive(HasField)]
pub struct Person {
    pub name: String,
    pub age: u8,
}
```

it produces, for each field, the field's type as the associated `Value` and a body that simply borrows it:

```rust
impl HasField<Symbol!("name")> for Person {
    type Value = String;

    fn get_field(&self, key: PhantomData<Symbol!("name")>) -> &Self::Value {
        &self.name
    }
}

impl HasFieldMut<Symbol!("name")> for Person {
    fn get_field_mut(&mut self, key: PhantomData<Symbol!("name")>) -> &mut Self::Value {
        &mut self.name
    }
}

// and the same pair for `age`, with `Value = u8`
```

The `PhantomData<Tag>` parameter carries no value. It exists so a call site can say *which* field it means
when several `HasField` impls are in scope, which is what `PhantomData::<Symbol!("name")>` at a call site is
doing. [`HasFieldMut`](../traits/has_field.md) extends `HasField` with `get_field_mut`; it is always
generated alongside the read accessor, whether or not anything uses it.

A tuple struct produces the same shape with a positional tag:

```rust
impl HasField<Index<0>> for Rectangle {
    type Value = f64;

    fn get_field(&self, key: PhantomData<Index<0>>) -> &Self::Value {
        &self.0
    }
}
```

A generic struct has its parameters split into impl-position generics, type arguments, and a `where` clause
and threaded onto every impl, so `struct Wrapper<T> { value: T }` yields
`impl<T> HasField<Symbol!("value")> for Wrapper<T>` with `Value = T`. A struct lifetime carries through the
same way, and a borrowed field type is kept verbatim as `Value`.

**The smart-pointer behaviour comes from the library, not the derive.** `HasField` and `HasFieldMut` carry
blanket impls for any type whose [`Deref`](https://doc.rust-lang.org/std/ops/trait.Deref.html) target
implements them, which is what lets `Box<Person>` resolve a field read to the inner struct. Those blanket
impls are marked so the compiler does not suggest them in a diagnostic, which keeps a missing-field error
pointed at the struct that lacks the field rather than at the pointer.

Each generated impl is also aimed at the field it came from, so a compiler error about one field's access —
a conflict with a hand-written impl, or the "but the trait is implemented for" hint inside a missing-field
error — underlines that field rather than the whole `#[derive(HasField)]`.

## Gotchas

**A unit struct produces nothing, silently.** There is no field to key, so the derive succeeds and emits no
impls. That is correct, but it means the mistake surfaces later, as an unsatisfied `HasField` bound at a
wiring site rather than as a complaint on the struct.

**Two spellings of a field name are unrelated types.** `Symbol!("first_name")` and `Symbol!("firstName")`
have nothing to do with each other, and a mismatch is reported as a missing `HasField` bound rather than as
a typo. This is the usual cause of a read that "should" work.

**A raw-identifier field is tagged without the `r#`.** The tag for `r#type` is `Symbol!("type")`, so
`Symbol!("r#type")` matches nothing.

**A tuple field is keyed by `Index<N>`, never by a `Symbol!` of the number.** `Index<0>` and `Symbol!("0")`
are different types, and only the first is generated.

**The mutable accessor is always generated.** There is no way to derive read-only access. If a field must
not be mutated by an implementation, that is a matter of what the implementations declare rather than
something this derive can restrict.

**It does not accept an enum.** `#[derive(HasField)]` parses a struct. The plural
[`#[derive(HasFields)]`](./derive_has_fields.md) is the one that takes both, and the near-identical names
are easy to confuse.

## Related constructs

- [`#[derive(HasFields)]`](./derive_has_fields.md) — the whole-shape counterpart, commonly derived
  alongside this one.
- [`HasField`](../traits/has_field.md) — the trait this generates, with `HasFieldMut` and the provider-side
  `FieldGetter`.
- [`Symbol!`](../macros/symbol.md) — the tag for a named field.
- [`Index`](../types/index.md) — the tag for a tuple field.
- [`#[implicit]`](../attributes/implicit.md) — the idiomatic way to read a field, generating the bound for
  you.
- [`#[cgp_auto_getter]`](../macros/cgp_auto_getter.md) — a named accessor over the same access.
- [`UseField`](../providers/use_field.md) — where a field name becomes a wiring decision.
- [`ChainGetters`](../providers/chain_getters.md) — reaching a field on a nested context.
- [`#[derive(CgpData)]`](./derive_cgp_data.md) — the umbrella derive, which includes this one.

The ideas behind it:

- [Impl-side dependencies](/docs/concepts/impl-side-dependencies) — why a field requirement belongs on the
  implementation rather than in the interface.
- [Extensible records](/docs/concepts/extensible-records) — treating a struct as a product of named fields.

## Source

- Entry point: [`derive_has_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/derive_has_field.rs)
- Codegen: [`cgp_data/derive_has_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/cgp_data/derive_has_field.rs)
- Traits: [`has_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/has_field.rs)
  and [`has_field_mut.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/has_field_mut.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
