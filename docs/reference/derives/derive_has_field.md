---
sidebar_label: '#[derive(HasField)]'
sidebar_position: 1
---

# `#[derive(HasField)]`

`#[derive(HasField)]` generates shared and mutable field accessors keyed by type-level tags.

## Overview

A generic implementation cannot access fields with expressions such as `self.name`. It runs against
a **context**, the type that supplies the values it needs as fields, but does not know that context's
concrete type.

`#[derive(HasField)]` lets the implementation request a field through a trait bound. It gives each
struct field a type-level tag, so the implementation can ask for a `String` field named `name`
without knowing the struct:

```rust
Self: HasField<Symbol!("name"), Value = String>
```

Any struct deriving `HasField` with a matching name and field type satisfies that bound.
[`Symbol!("name")`](../macros/symbol.md) is a type, so the compiler resolves the tag to a direct field
access without a runtime string lookup.

Higher-level field access constructs generate the `HasField` bound for you. An
[`#[implicit]`](../attributes/implicit.md) argument, a
[`#[cgp_auto_getter]`](../macros/cgp_auto_getter.md) method, or a
[`UseField`](../providers/use_field.md) wiring entry uses a field name to obtain the value. The struct
supplies the corresponding implementations by deriving `HasField`.

## Usage

Apply `HasField` to a struct without arguments or helper attributes:

```rust
#[derive(HasField)]
pub struct Person {
    pub name: String,
    pub age: u8,
}
```

It accepts any struct shape. The shapes differ only in how each field's tag is computed.

### Named fields

A named field is keyed by [`Symbol!`](../macros/symbol.md), the type-level string of its identifier. The
struct above gains access keyed by `Symbol!("name")` and `Symbol!("age")`.

A field written as a [raw identifier](https://doc.rust-lang.org/reference/identifiers.html) is keyed by
its *logical* name, with the `r#` stripped: a field `r#type` is keyed by `Symbol!("type")`, not
`Symbol!("r#type")`. The generated accessor still borrows the real field.

### Tuple fields

Tuple fields use [`Index<N>`](../types/index_type.md) tags, where `N` is the field's zero-based
position:

```rust
#[derive(HasField)]
pub struct Rectangle(pub f64, pub f64);
```

`Rectangle` gains access keyed by `Index<0>` and `Index<1>`. A tuple field is never keyed by a `Symbol!`
of its position, because `Index<0>` and `Symbol!("0")` are different types.

### Unit structs

A unit struct is accepted without generating accessors because it has no fields:

```rust
#[derive(HasField)]
pub struct App;
```

A fieldless context can still supply component choices without storing values. Deriving `HasField`
on it is harmless, but does not satisfy a field-access bound. Code requiring a field will report an
unsatisfied bound at the use site.

### Generic structs

The generated implementations preserve generic parameters, lifetimes, and the struct's `where`
clause:

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
[`#[derive(HasFields)]`](./derive_has_fields.md) (note the plural), and the two are frequently derived
together.

## Examples

An implementation declares the field it needs as a bound. Here, `GreetHello` requires a `String`
field named `name`:

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

`Person` supplies the field access and selects `GreetHello` as its implementation:

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

`Person` satisfies the field bound required by `GreetHello`, so `person.greet()` prints its name.
Removing the `name` field prevents the call from compiling because the required `HasField`
implementation is missing.

An [`#[implicit]`](../attributes/implicit.md) argument expresses the field dependency as a function
parameter:

```rust
#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self, #[implicit] name: &str) {
        println!("Hello, {name}!");
    }
}
```

The implicit form generates a `HasField` bound and reads the field for you. Prefer it for this
ordinary field read; the explicit form is useful when reading generated code or diagnostics.

Field access passes through smart pointers without an additional derive. `Box<Person>` and
newtypes that dereference to `Person` can read its fields through the library's blanket implementations.

## When to use it

Derive `HasField` when implementations need to read a context's fields through CGP's field-access
interface. A context can still select implementations and implement consumer traits without the
derive, but its fields need corresponding trait implementations to support generic reads.

Choose the field-access syntax according to what the implementation needs:

- **[`#[implicit]`](../attributes/implicit.md)**: use by default to read a field from the
  implementation's own context as a parameter, without declaring a getter trait.
- **[`#[cgp_auto_getter]`](../macros/cgp_auto_getter.md)**: use when the read should be a named
  capability, the field belongs to another type, or the getter needs a type inferred from the field.
- **A hand-written `HasField` bound**: use when the higher-level forms do not fit. Add `Value = T` to
  constrain the field type.
- **[`#[derive(HasFields)]`](./derive_has_fields.md)**: use for processing the whole structure, such
  as serialization or generic field traversal. Derive both when code also needs individual reads.

Incremental construction requires [`BuildField`](./derive_build_field.md). `HasField` provides access
to existing fields; [`CgpData`](./derive_cgp_data.md) combines it with the representation and builder
needed for an extensible record.

## Under the hood

The derive leaves the struct definition untouched and adds **two impls per field**: a read accessor and a
mutable one. From this input:

```rust
#[derive(HasField)]
pub struct Person {
    pub name: String,
    pub age: u8,
}
```

Each generated accessor uses the declared field type as `Value` and returns a reference to the field:

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

`PhantomData<Tag>` selects the field without storing a runtime tag value. A call such as
`get_field(PhantomData::<Symbol!("name")>)` selects the `name` accessor.
[`HasFieldMut`](../traits/field-access/has_field_mut.md) extends `HasField` with `get_field_mut`, and
the derive always generates it alongside shared access.

A tuple struct produces the same shape with a positional tag:

```rust
impl HasField<Index<0>> for Rectangle {
    type Value = f64;

    fn get_field(&self, key: PhantomData<Index<0>>) -> &Self::Value {
        &self.0
    }
}
```

A generic struct has its parameters split into impl-position generics, type arguments, and a `where`
clause and threaded onto every impl, so `struct Wrapper<T> { value: T }` yields
`impl<T> HasField<Symbol!("value")> for Wrapper<T>` with `Value = T`. A struct lifetime carries through
the same way, and a borrowed field type is kept verbatim as `Value`.

The library supplies smart-pointer access through blanket implementations of `HasField` and
`HasFieldMut`. Shared access requires [`Deref`](https://doc.rust-lang.org/std/ops/trait.Deref.html),
and mutable access requires `DerefMut`, with the target implementing the corresponding field trait.
The blanket implementations are marked to keep the compiler from recommending them in missing-field
diagnostics, directing attention to the underlying struct.

Each generated impl is also aimed at the field it came from, so a compiler error about one field's access
underlines that field rather than the whole `#[derive(HasField)]`. That covers a conflict with a
hand-written impl, and the "but the trait is implemented for" hint inside a missing-field error.

## Common Mistakes

**A unit struct does not generate field-access implementations.** The derive succeeds, but a later
field read reports an unsatisfied `HasField` bound.

**Field tags must match exactly.** `Symbol!("first_name")` and `Symbol!("firstName")` are different
types. A spelling mismatch produces an unsatisfied `HasField` bound.

**A raw-identifier field is tagged without the `r#`.** The tag for `r#type` is `Symbol!("type")`, so
`Symbol!("r#type")` matches nothing.

**A tuple field is keyed by `Index<N>`, never by a `Symbol!` of the number.** `Index<0>` and
`Symbol!("0")` are different types, and only the first is generated.

**The mutable accessor is always generated.** The derive does not offer a read-only mode. Limit
mutation through the access bounds and references supplied to implementations.

**It does not accept an enum.** `#[derive(HasField)]` parses a struct. The plural
[`#[derive(HasFields)]`](./derive_has_fields.md) is the one that takes both, and the near-identical names
are easy to confuse.

## Related constructs

These references cover the related derives, generated traits, and supporting types:

- [`#[derive(HasFields)]`](./derive_has_fields.md) — the whole-shape counterpart, commonly derived
  alongside this one.
- [`HasField`](../traits/field-access/has_field.md) — the trait this generates, with `HasFieldMut` and the provider-side
  `FieldGetter`.
- [`Symbol!`](../macros/symbol.md) — the tag for a named field.
- [`Index`](../types/index_type.md) — the tag for a tuple field.
- [`#[implicit]`](../attributes/implicit.md) — the idiomatic way to read a field, generating the bound for
  you.
- [`#[cgp_auto_getter]`](../macros/cgp_auto_getter.md) — a named accessor over the same access.
- [`UseField`](../providers/use_field.md) — where a field name becomes a wiring decision.
- [`ChainGetters`](../providers/chain_getters.md) — reaching a field on a nested context.
- [`#[derive(CgpData)]`](./derive_cgp_data.md) — the umbrella derive, which includes this one.

The ideas behind it are explained on these concept pages:

- [Impl-side dependencies](/docs/concepts/impl-side-dependencies) — why a field requirement belongs on the
  implementation rather than in the interface.
- [Extensible records](/docs/concepts/extensible-records) — treating a struct as a product of named fields.

## Source

The implementation is defined in these source files:

- Entry point: [`derive_has_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-lib/src/derive_has_field.rs)
- Codegen: [`cgp_data/derive_has_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/cgp_data/derive_has_field.rs)
- Traits: [`has_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/has_field.rs)
  and [`has_field_mut.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/has_field_mut.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
