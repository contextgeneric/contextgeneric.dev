---
sidebar_label: 'FieldGetter'
---

# `FieldGetter`

The provider-side mirror of `HasField` — field access that gets wired.

## What it's for

[`HasField`](./has_field.md) is a bound an implementation states about its own context. Sometimes field
access should instead be **chosen by wiring**, so that a context decides which of its fields answers a
given getter. That needs the provider-trait shape: the context as an explicit type argument rather than
as `&self`.

`FieldGetter` is that shape:

```rust
pub trait FieldGetter<Context, Tag> {
    type Value;

    fn get_field(context: &Context, _tag: PhantomData<Tag>) -> &Self::Value;
}
```

`Self` is the **provider** — a zero-sized marker carrying no data — while `Context` is the type the field
is read from. This is the ordinary consumer/provider duality applied to field access: **you bound against
[`HasField`](./has_field.md), and you wire `FieldGetter`**. They are not alternatives.

[`UseField`](../providers/use_field.md) is the provider that implements it by reading a named field, and
it is what [`#[cgp_getter]`](../macros/cgp_getter.md) targets.

## Using it

**It is in the prelude**, so `use cgp::prelude::*;` is enough.

You rarely name the trait directly. What you write is a wiring entry naming a provider that implements
it:

```rust
delegate_components! {
    Person {
        NameGetterComponent: UseField<Symbol!("first_name")>,
    }
}
```

Reading it back: the getter component is answered by [`UseField`](../providers/use_field.md), which
implements `FieldGetter` by reading the `first_name` field — a field whose name need not match the
getter's method name, which is the whole point of wiring the access rather than deriving it.

Writing a `FieldGetter` impl by hand is the escape hatch for a getter whose value is computed rather than
stored, and it is an ordinary trait impl on a marker type of your own.

## Examples

The wired form, where the field's name and the getter's name differ:

```rust
use cgp::prelude::*;

#[cgp_getter(NameGetter)]
pub trait HasName {
    fn name(&self) -> &String;
}

#[derive(HasField)]
pub struct Person {
    pub first_name: String,
}

delegate_components! {
    Person {
        NameGetterComponent: UseField<Symbol!("first_name")>,
    }
}
```

`person.name()` now returns the `first_name` field. **Value context, self-targeted.**

A second context can answer the same getter from a different field by changing one wiring line, which is
what the provider-side shape buys and what a plain [`HasField`](./has_field.md) bound cannot express.

## When to reach for it, and when not

**Wire a `FieldGetter` only when a context must choose which field a getter reads**, which is the
advanced case. Everything else is better served higher up.

- **Use an [`#[implicit]`](../attributes/implicit.md) argument** for a field an implementation reads from
  its own context. It generates a [`HasField`](./has_field.md) bound and involves no provider at all.
- **Use [`#[cgp_auto_getter]`](../macros/cgp_auto_getter.md)** when the read must be a named capability,
  keyed by the method name. It generates a blanket impl over `HasField` and needs no wiring.
- **Use [`#[cgp_getter]`](../macros/cgp_getter.md)** when the source field must be chosen per context.
  That is what makes the getter a full component with a `FieldGetter` provider trait, and it costs a
  wiring line per context.
- **Reach for [`MutFieldGetter`](./mut_field_getter.md)** when the wired access must also mutate.

Do not treat it as an alternative to [`HasField`](./has_field.md). One is what an implementation
requires; the other is what a context supplies.

## Under the hood

:::note

### Advanced

This section shows the impl that connects wiring to a context's own fields.

:::

The provider side connects to ordinary derived field access through one blanket impl:
[`UseContext`](../providers/use_context.md) implements `FieldGetter` for any context that already has the
field, delegating straight through.

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

So wiring a getter to `UseContext` means "read the field of the same name off the context", which is the
behaviour [`#[cgp_auto_getter]`](../macros/cgp_auto_getter.md) gives without any wiring.

[`UseField<Tag>`](../providers/use_field.md) is the more interesting provider: it fixes the tag at the
wiring site, so the field it reads is decided there rather than by the method's name.

The lifetime-safe variant [`FieldMapper`](./field_mapper.md) is a blanket impl over every `FieldGetter`
with the getter and tag `'static`, which is what [`ChainGetters`](../providers/chain_getters.md) relies
on.

## Gotchas

**`Self` is the provider, not the context.** The context is the first type parameter. Reading the
signature the other way round is the usual confusion when meeting a provider trait, and it applies to
every provider trait rather than to this one specially.

**It is not an alternative to [`HasField`](./has_field.md).** Bounding an implementation on `FieldGetter`
instead of `HasField` is almost always a mistake: the implementation wants a requirement on its context,
not a provider parameter.

**The getter's method name and the field's name are independent once wired.** That is the feature, and it
means a reader cannot infer the field from the getter — the wiring line is the only place the answer
lives.

**A getter wired to [`UseContext`](../providers/use_context.md) needs the field to exist under the
getter's own name**, since that impl keys on the tag it is given.

**It returns a reference.** A getter that must produce an owned or computed value uses
[`MRef`](../types/mref.md) as its return type instead.

## Related constructs

- [`HasField`](./has_field.md) — the consumer side you bound against.
- [`MutFieldGetter`](./mut_field_getter.md) — the mutable extension of this trait.
- [`FieldMapper`](./field_mapper.md) — the lifetime-safe form, for nested access.
- [`UseField`](../providers/use_field.md) — the provider that implements it by reading a named field.
- [`UseContext`](../providers/use_context.md) — the provider that routes back through the context's own
  fields.
- [`#[cgp_getter]`](../macros/cgp_getter.md) — the macro that makes a getter a full component.
- [`#[cgp_auto_getter]`](../macros/cgp_auto_getter.md) — the simpler getter, keyed by method name.
- [`ChainGetters`](../providers/chain_getters.md) — composing getters to reach a nested field.
- [`MRef`](../types/mref.md) — the owned-or-borrowed return type for a getter that may produce its value.

The ideas behind it:

- [Consumer and provider traits](/docs/concepts/consumer-and-provider-traits) — the duality this trait is
  the field-access instance of.

## Source

- [`has_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/has_field.rs)
  — `FieldGetter` and the `UseContext` impl
- [`use_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/impls/use_field.rs)
  — the `UseField` provider

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
