---
sidebar_label: 'HasField'
---

# `HasField`

Tag-keyed field access, with `HasFieldMut` and the provider-side `FieldGetter`.

## What it's for

An implementation written against a **context** — the type the capability runs against, which supplies the
values it needs as its fields — is generic over that context and cannot name its concrete type. So it cannot
write `self.name`. Yet reading a value out of the context is the commonest thing such an implementation does.

`HasField<Tag>` is the trait that closes the gap. The field's *name* becomes a type, and the implementation
asks for it as an ordinary trait bound:

```rust
Self: HasField<Symbol!("name"), Value = String>
```

Any context with a matching field satisfies that bound. Nothing is compared by string at runtime —
[`Symbol!("name")`](../macros/symbol.md) is a type, so the compiler resolves which field is meant and the read
compiles to a direct field access.

This is the foundation the whole ergonomic surface stands on. An
[`#[implicit]`](../attributes/implicit.md) argument, a [`#[cgp_auto_getter]`](../macros/cgp_auto_getter.md)
method, and a [`UseField`](../providers/use_field.md) wiring entry all generate this bound from a name you
already wrote. **You will read `HasField` far more often than you write it**, and the impls themselves come
from [`#[derive(HasField)]`](../derives/derive_has_field.md).

## Using it

Everything on this page except `MapField` and `FieldMapper` is in the prelude, so `use cgp::prelude::*;` is
enough for the common cases. The family divides into a consumer side you bound against and a provider side
that gets wired.

### The consumer side

`HasField<Tag>` carries the field's type as an associated `Value` and hands back a reference:

```rust
pub trait HasField<Tag> {
    type Value;

    fn get_field(&self, _tag: PhantomData<Tag>) -> &Self::Value;
}
```

The `PhantomData<Tag>` argument carries nothing. It exists so a call site can say *which* field it means when
several impls are in scope, which is why reads are written `self.get_field(PhantomData::<Symbol!("name")>)`.

A named field is keyed by [`Symbol!("field_name")`](../macros/symbol.md); a tuple field by
[`Index<N>`](../types/index.md).

`HasFieldMut<Tag>` is the mutable extension, a supertrait of `HasField<Tag>`:

```rust
pub trait HasFieldMut<Tag>: HasField<Tag> {
    fn get_field_mut(&mut self, tag: PhantomData<Tag>) -> &mut Self::Value;
}
```

Both carry diagnostic notes pointing a reader at
[`#[derive(HasField)]`](../derives/derive_has_field.md) when the bound is unsatisfied, so a missing field reads
as a missing derive rather than an opaque trait failure.

### The provider side

Field access can be *wired* rather than only implemented on the context, which needs the provider-trait shape:
the context as an explicit argument instead of `&self`.

```rust
pub trait FieldGetter<Context, Tag> {
    type Value;

    fn get_field(context: &Context, _tag: PhantomData<Tag>) -> &Self::Value;
}

pub trait MutFieldGetter<Context, Tag>: FieldGetter<Context, Tag> {
    fn get_field_mut(context: &mut Context, tag: PhantomData<Tag>) -> &mut Self::Value;
}
```

This is the ordinary consumer/provider duality: you *bound* against `HasField`, and you *wire*
`FieldGetter`. [`UseField`](../providers/use_field.md) is the provider that implements it by reading a named
field, and it is what [`#[cgp_getter]`](../macros/cgp_getter.md) targets.

### The lifetime helpers

`MapField` and `FieldMapper` are the two members **not** in the prelude — import them from
`cgp::core::field::traits`. They exist for one reason, and it is a lifetime-inference problem rather than a
capability: chaining `context.get_field().get_field()` would force the intermediate `Value` to be `'static`.
Passing a closure instead lets the compiler infer the right lifetime:

```rust
pub trait MapField<Tag>: HasField<Tag> {
    fn map_field<T>(
        &self,
        _tag: PhantomData<Tag>,
        mapper: impl for<'a> FnOnce(&'a Self::Value) -> &'a T,
    ) -> &T;
}

pub trait FieldMapper<Context, Tag>: FieldGetter<Context, Tag> {
    fn map_field<T>(
        context: &Context,
        _tag: PhantomData<Tag>,
        mapper: impl for<'a> FnOnce(&'a Self::Value) -> &'a T,
    ) -> &T;
}
```

You will not normally call either. [`ChainGetters`](../providers/chain_getters.md) is what uses `map_field`,
which is how a getter reaches a field on a nested context.

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

`Person` derives the access, so it satisfies exactly the bound `GreetHello` requires and the wiring compiles.

**Written idiomatically, none of that bound is visible.** The same read is an
[`#[implicit]`](../attributes/implicit.md) argument:

```rust
#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self, #[implicit] name: &str) {
        println!("Hello, {name}!");
    }
}
```

which generates the identical bound and the identical read. The explicit form above is what you meet in an
error message or an expansion, not what you type.

Field access also passes through smart pointers with no extra work, because the trait carries a `Deref`
forwarding impl:

```rust
let boxed: Box<Person> = Box::new(Person { name: "Alice".to_owned() });

assert_eq!(boxed.get_field(PhantomData::<Symbol!("name")>), "Alice");
```

## When to reach for it, and when not

**Bound against `HasField` only when the ergonomic constructs cannot do the job**, which is rarely. The
ordering is settled and worth following.

- **Use an [`#[implicit]`](../attributes/implicit.md) argument by default.** It reads a field of the
  implementation's own context as a plain parameter, generating this bound for you. It covers the common case,
  including a field several implementations each read.
- **Use [`#[cgp_auto_getter]`](../macros/cgp_auto_getter.md)** when the read must be a *named* capability,
  when the field lives on a type other than the context, or when the getter should carry a type inferred from
  the field — the three cases an implicit argument cannot reach.
- **Use [`#[cgp_getter]`](../macros/cgp_getter.md) with [`UseField`](../providers/use_field.md)** only when a
  context needs to choose *which* field the getter reads. That is the advanced case and costs a wiring line
  per context.
- **Write the bound by hand** when none of those fit — a bound on a type that is not `Self`, say. It is an
  ordinary trait bound, and `Value = T` pins the field's type.

Two neighbours are easy to confuse with it. [`HasFields`](./has_fields.md) — plural — is the whole-shape view,
for code that must process *every* field rather than one named one; the two are complementary and often
derived together. And `FieldGetter` is not an alternative to `HasField` but its provider-side mirror: you do
not choose between them, you bound against one and wire the other.

## Under the hood

:::note

### Advanced

This section shows the blanket impls that make the access compose. You do not need it to read a field, but a
missing field is reported against the generated bound with the tag fully expanded, so recognizing the shape
makes that error legible.

:::

The per-field impls come almost entirely from
[`#[derive(HasField)]`](../derives/derive_has_field.md). What the trait module itself supplies is the blanket
impls that let the access compose, and there are four worth knowing.

**Smart-pointer forwarding.** `HasField` is implemented for any type whose
[`Deref`](https://doc.rust-lang.org/std/ops/trait.Deref.html) target implements it, so a `Box<Person>` or a
newtype resolves a read to the inner struct. `HasFieldMut` carries the analogous `DerefMut` impl, with the
target additionally bounded `'static`. Both are marked so the compiler does **not** suggest them in a
diagnostic — which is what keeps a missing-field error pointed at the struct that lacks the field rather than
at the pointer.

**`UseContext` as a field getter.** The provider side connects to wiring through one impl: `UseContext`
implements `FieldGetter` for any context that already has the field, delegating straight through.

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

**The lifetime helpers are free.** `MapField` is a blanket impl for every `HasField` whose tag is `'static`,
and `FieldMapper` for every `FieldGetter` with the getter and tag `'static`. So every getter gains
`map_field` without anyone implementing it, which is what [`ChainGetters`](../providers/chain_getters.md)
relies on.

## Gotchas

**Two spellings of a name are unrelated types.** `Symbol!("first_name")` and `Symbol!("firstName")` have
nothing to do with each other, and a mismatch reports as a missing `HasField` bound rather than as a typo.
This is the usual cause of a read that "should" work.

**`MapField` and `FieldMapper` are not in the prelude.** Import them from `cgp::core::field::traits`. Every
other trait on this page is in the prelude.

**A tuple field is keyed by [`Index<N>`](../types/index.md), never by a `Symbol!` of the number.** `Index<0>`
and `Symbol!("0")` are different types.

**The mutable accessor is not separately opt-in.** `#[derive(HasField)]` always emits `HasFieldMut` beside
`HasField`, so there is no derive-level way to make a field read-only. Restricting mutation is a matter of
what the implementations declare.

**`Value` is an associated type, so it can be pinned or left open.** Writing
`HasField<Symbol!("name")>` alone accepts a field of any type; adding `Value = String` requires that type.
Omitting it when you meant to pin it produces an inference failure further along rather than at the bound.

**The `PhantomData` argument is required and carries the tag.** `self.get_field(PhantomData)` works only where
inference can determine the tag from context; at an ambiguous site write `PhantomData::<Symbol!("name")>`.

## Related constructs

- [`#[derive(HasField)]`](../derives/derive_has_field.md) — generates the per-field impls; what a context
  writes.
- [`HasFields`](./has_fields.md) — the plural, whole-shape counterpart.
- [`Symbol!`](../macros/symbol.md) and [`Index`](../types/index.md) — the tags that key a field.
- [`#[implicit]`](../attributes/implicit.md) — the idiomatic way to read a field.
- [`#[cgp_auto_getter]`](../macros/cgp_auto_getter.md) and [`#[cgp_getter]`](../macros/cgp_getter.md) — getter
  traits over the same access.
- [`UseField`](../providers/use_field.md) — the provider-side implementation of `FieldGetter`.
- [`ChainGetters`](../providers/chain_getters.md) — composes getters through `map_field` to reach a nested
  field.
- [`HasBuilder`](./has_builder.md) — where `HasField` reappears on a partial record, gated on presence.

The ideas behind it:

- [Impl-side dependencies](/docs/concepts/impl-side-dependencies) — why a field requirement belongs on the
  implementation rather than the interface.
- [Implicit arguments](/docs/concepts/implicit-arguments) — the ergonomic surface built on this trait.

## Source

- [`has_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/has_field.rs)
  — `HasField`, `FieldGetter`, the `Deref` forwarding, and the `UseContext` impl
- [`has_field_mut.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/has_field_mut.rs)
  — `HasFieldMut`, `MutFieldGetter`, the `DerefMut` forwarding
- [`map_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/map_field.rs)
  — `MapField` and `FieldMapper`
- [`use_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/impls/use_field.rs)
  — the `UseField` provider

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
