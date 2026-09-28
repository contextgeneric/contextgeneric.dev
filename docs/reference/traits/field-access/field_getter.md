---
title: 'FieldGetter — field access as a provider'
description: 'The provider-side form of HasField: a getter component takes any FieldGetter provider, such as UseField, through WithProvider.'
sidebar_label: 'FieldGetter'
sidebar_position: 3
---

# `FieldGetter`

The provider-side form of `HasField`: field access a context chooses by wiring.

:::info

### Generated machinery

**You are not expected to name `FieldGetter` in wiring.** A getter component defined with
[`#[cgp_getter]`](../../macros/cgp_getter.md) accepts any provider of it through
[`WithProvider`](../../providers/with_provider.md), and
[`UseField`](../../providers/use_field.md) is the provider you usually name there, as
`WithField<Tag>`. What you write is the wiring entry. The one case for implementing the trait
yourself is a getter provider whose read no existing provider expresses.

:::

## Overview

[`HasField`](./has_field.md) is a bound an implementation states about its own context. Sometimes
the field should instead be **chosen by wiring**, so that each
[**context**](/docs/reference/glossary#context), the type the method runs on, which supplies the
values it needs as its fields, decides which of its fields answers a getter. That needs the
provider-trait shape: a zero-sized provider as `Self`, and the context as an explicit type argument.

`FieldGetter` is that shape, and it is the ordinary consumer and provider split applied to field
access: you bound against [`HasField`](./has_field.md), and you wire a `FieldGetter`. Four library
providers implement it. `UseField<Tag>` reads the field `Tag`;
[`UseFieldRef`](../../providers/use_field_ref.md) reads one through `AsRef`;
[`UseContext`](../../providers/use_context.md) reads the field named by the tag it is asked under;
and [`ChainGetters`](../../providers/chain_getters.md) composes others to reach a nested field.

## Definition

`FieldGetter<Context, Tag>` mirrors [`HasField`](./has_field.md), with the context as an explicit
type argument instead of `&self`:

```rust
pub trait FieldGetter<Context, Tag> {
    type Value;

    fn get_field(context: &Context, _tag: PhantomData<Tag>) -> &Self::Value;
}
```

`Self` is the provider, a marker carrying no data, and `Context` is the type the field is read from.
`Tag` is the tag the provider is asked under: through a getter component, that is the component's
own marker. `Value` is the field's type, and `get_field` returns a borrow of it. It is a plain trait
rather than a component provider trait, so its impls carry no
[`IsProviderFor`](../wiring/is_provider_for.md) pair.

## Usage

It is in the prelude, so `use cgp::prelude::*;` is enough. A getter component takes a `FieldGetter`
provider wrapped in `WithProvider`, most often through the `WithField<Tag>` alias for
`WithProvider<UseField<Tag>>`, which is imported from `cgp::core::field::impls`:

```rust
use cgp::core::field::impls::WithField;

delegate_components! {
    Person {
        NameGetterComponent: WithField<Symbol!("first_name")>,
    }
}
```

The getter then reads `first_name`, whose name need not match the getter's method. A bare
`UseField<Symbol!("first_name")>` entry works as well, through a separate impl that `#[cgp_getter]`
emits for `UseField` directly, without going through `FieldGetter`.

## Examples

One getter component answered on two contexts by two `FieldGetter` providers, `UseField` reading a
differently named field and a hand-written provider reading a nested one:

```rust
use cgp::prelude::*;
use cgp::core::field::impls::WithField;

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
        NameGetterComponent: WithField<Symbol!("first_name")>,
    }
}

pub struct Profile {
    pub display_name: String,
}

#[derive(HasField)]
pub struct Account {
    pub profile: Profile,
}

pub struct ReadDisplayName;

impl<Context, Tag> FieldGetter<Context, Tag> for ReadDisplayName
where
    Context: HasField<Symbol!("profile"), Value = Profile>,
{
    type Value = String;

    fn get_field(context: &Context, _tag: PhantomData<Tag>) -> &String {
        &context.get_field(PhantomData).display_name
    }
}

delegate_components! {
    Account {
        NameGetterComponent: WithProvider<ReadDisplayName>,
    }
}

check_components! {
    Person {
        NameGetterComponent,
    }
}

check_components! {
    Account {
        NameGetterComponent,
    }
}

pub fn demo() {
    let person = Person {
        first_name: "Ada".to_owned(),
    };
    let account = Account {
        profile: Profile {
            display_name: "ada99".to_owned(),
        },
    };

    assert_eq!(person.name(), "Ada");
    assert_eq!(account.name(), "ada99");
}
```

`ReadDisplayName` ignores the tag it is asked under and reads a field of a field, so it is an
ordinary impl on a marker type, with no attribute. Both contexts are
[value contexts](/docs/reference/glossary#value-context), and the getter is
[self-targeted](/docs/reference/glossary#self-targeted-component).

## When to use it

**Wire a `FieldGetter` only when a context must choose which field a getter reads**, the advanced
case. The simpler forms come first:

- **An [`#[implicit]`](../../attributes/implicit.md) argument** reads a field of the
  implementation's own context, and involves no provider.
- **[`#[cgp_auto_getter]`](../../macros/cgp_auto_getter.md)** makes the read a named trait keyed by
  the method name, with no wiring.
- **[`#[cgp_getter]`](../../macros/cgp_getter.md)** makes the getter a component, whose wiring entry
  names the `FieldGetter` provider: `WithField<Tag>` for a field of the context, or
  `WithProvider<ChainGetters<…>>` for a field of a nested value.

Write a provider of your own only for a read those do not cover. A nested field is usually a
`ChainGetters` list of `UseField` steps rather than a hand-written provider like the example's.

## Under the hood

For a one-method getter, `#[cgp_getter]` emits an impl of the getter's provider trait for
`WithProvider<P>`, bounded on `P: FieldGetter` under the component's marker. `cargo cgp expand` on
the example shows:

```rust
impl<__Context__, __Provider__> NameGetter<__Context__> for WithProvider<__Provider__>
where
    __Provider__: FieldGetter<__Context__, NameGetterComponent, Value = String>,
{
    fn name(__context__: &__Context__) -> &String {
        __Provider__::get_field(
            __context__,
            ::core::marker::PhantomData::<NameGetterComponent>,
        )
    }
}
```

`UseField<Tag>` implements `FieldGetter` for every tag it is asked under, reading its own `Tag`
instead, which is why it serves any getter:

```rust
impl<Context, OutTag, Tag, Value> FieldGetter<Context, OutTag> for UseField<Tag>
where
    Context: HasField<Tag, Value = Value>,
{
    type Value = Value;

    fn get_field(context: &Context, _tag: PhantomData<OutTag>) -> &Value {
        context.get_field(PhantomData)
    }
}
```

`UseContext` reads the tag it is asked under, so through
[`WithContext`](../../providers/with_context.md) it needs the context to implement
`HasField<NameGetterComponent>`, keyed by the component rather than by a field name. The
lifetime-safe [`FieldMapper`](./field_mapper.md) is a blanket impl over every `FieldGetter`, which
`ChainGetters` relies on.

## Common Mistakes

**A `FieldGetter` provider does not answer a getter on its own.** Wiring
`NameGetterComponent: ReadDisplayName`, without `WithProvider`, names a provider that implements
`FieldGetter` but not the getter's provider trait, so the check fails on its missing marker:

```text
error[E0277]: the trait bound `ReadDisplayName: IsProviderFor<NameGetterComponent, Account>` is not satisfied
```

Wrap it, as `WithProvider<ReadDisplayName>`.

**`Self` is the provider, not the context.** The context is the first type parameter, which is the
usual confusion when meeting any provider trait.

**It is not an alternative to [`HasField`](./has_field.md).** An implementation that needs a field
bounds its context on `HasField`; `FieldGetter` is what a context supplies through wiring.

**The getter's method name and the field's name are independent once wired.** That is the feature,
and it means a reader cannot infer the field from the getter: the wiring line holds the answer.

## Related constructs

- [`HasField`](./has_field.md): the consumer side you bound against.
- [`MutFieldGetter`](./mut_field_getter.md): the mutable extension.
- [`FieldMapper`](./field_mapper.md): the lifetime-safe form, for nested access.
- [`UseField`](../../providers/use_field.md) and [`WithField`](../../providers/with_field.md): the
  provider that reads a named field, and its `WithProvider` alias.
- [`UseContext`](../../providers/use_context.md): the provider that reads the field named by its
  tag.
- [`ChainGetters`](../../providers/chain_getters.md): composes getters to reach a nested field.
- [`#[cgp_getter]`](../../macros/cgp_getter.md): the macro whose getters take these providers.

The ideas behind it:

- [Consumer and provider traits](/docs/concepts/consumer-and-provider-traits): the split this trait
  is the field-access instance of.

## Source

- [`traits/has_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/has_field.rs):
  `FieldGetter` and the `UseContext` impl
- [`impls/use_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/impls/use_field.rs):
  the `UseField` impls

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
