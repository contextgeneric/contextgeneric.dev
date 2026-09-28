---
title: 'UseField — a getter that reads a named field'
description: 'The provider that implements a getter by reading the context field a type-level tag names, so the field name is a wiring choice, not the method name.'
sidebar_label: 'UseField'
sidebar_position: 3
---

# `UseField`

Implement a getter component by reading a context field named by a tag, so the field name is a wiring
decision rather than the method name.

## Overview

`UseField<Tag>` decouples a getter's method name from the field it reads. A getter component defined
with [`#[cgp_getter]`](../macros/cgp_getter.md) describes a value the context can supply, such as
`fn name(&self) -> &str`. The [**context**](/docs/reference/glossary#context) is the type the method runs on, which supplies
the values it needs as its fields. But the context may store the value under a different field name,
say `first_name`, and different contexts may store it under different names. `UseField<Tag>` carries
the field name as its type parameter, so wiring a getter to `UseField<Symbol!("first_name")>` makes it
read `first_name` even though the method is `name`. The field name lives in the wiring, not in the
trait.

This is the provider that [`#[cgp_getter]`](../macros/cgp_getter.md) targets. That macro generates a
`UseField` implementation for the getter's provider trait with the field tag left as a free parameter,
so a context picks the field by writing `UseField<Symbol!("...")>` in its wiring table. `UseField`
itself is the general provider underneath: it works for any tag the context's
[`HasField`](../traits/field-access/has_field.md) implementation supports.

The `Tag` is usually a [type-level string](/docs/reference/glossary#type-level-string) built with [`Symbol!`](../macros/symbol.md), such as
`Symbol!("name")`, or a type-level integer wrapped in `Index<N>` for a tuple field. These are the tags
that [`#[derive(HasField)]`](../derives/derive_has_field.md) produces `HasField` implementations for.
Like every CGP provider, `UseField<Tag>` carries no runtime value: it is a `PhantomData` marker named
in wiring, never constructed.

## Usage

`UseField` is in the prelude, so `use cgp::prelude::*;` is enough to name it. It takes one type
parameter, the field tag, and appears as the value of a getter component's wiring entry:

```rust
delegate_components! {
    Person {
        NameGetterComponent: UseField<Symbol!("first_name")>,
    }
}
```

The tag is a [`Symbol!`](../macros/symbol.md) for a named field or an `Index<N>` for a tuple field.
The context must have a `HasField` implementation for that tag, which
[`#[derive(HasField)]`](../derives/derive_has_field.md) supplies for every field of a struct.

`UseField<Tag>` also has an alias, [`WithField<Tag>`](with_field.md), which is
[`WithProvider<UseField<Tag>>`](with_provider.md). Both bind the same field; prefer the plain
`UseField<Tag>` form.

## Examples

The defining use wires a getter to a field whose name differs from the method, which is the case the
simpler [`#[cgp_auto_getter]`](../macros/cgp_auto_getter.md) cannot express. The method is `name`, but
the context stores the value in `first_name`:

```rust
use cgp::prelude::*;

#[cgp_getter]
pub trait HasName {
    fn name(&self) -> &str;
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

check_components! {
    Person {
        NameGetterComponent,
    }
}

pub fn demo() {
    let person = Person { first_name: "Alice".to_owned() };
    assert_eq!(person.name(), "Alice"); // reads the first_name field
}
```

`Person` is a [value context](/docs/reference/glossary#value-context): the getter reads its own
data. It wires `NameGetterComponent` to `UseField<Symbol!("first_name")>`, so `person.name()` reads
the `first_name` field. The method name and the field name diverge, and the field name comes
entirely from the wiring.

The same binding can be written with the [`WithField`](with_field.md) alias, which routes through
[`WithProvider`](with_provider.md).

## When to use it

**Reach for `UseField` only when the field name must differ from the method name**, or when a context
needs full control over which field a getter reads from. That is the advanced case
[`#[cgp_getter]`](../macros/cgp_getter.md) exists for, and `UseField` is the provider it is wired to.

For the common case of reading a field, prefer an [`#[implicit]`](../attributes/implicit.md) argument
instead, which reads a same-named field and looks like an ordinary function parameter with no getter
trait at all. Where a [getter trait](/docs/reference/glossary#getter-trait) is genuinely needed but the field name matches the method name, use
[`#[cgp_auto_getter]`](../macros/cgp_auto_getter.md), or wire [`UseFields`](use_fields.md), which keys
each method on its own name. The [`#[cgp_getter]`](../macros/cgp_getter.md) page works the choice
through in full.

`UseField` already handles the common borrowed-view getters: a `-> &str` getter over a `String` field
and a `-> &[u8]` getter over a `Vec<u8>` field work through `UseField`, because the generated
implementation borrows through `as_str()` or `as_ref()` based on the return type. Two neighbours cover
the cases it does not. When a getter returns `&T` and the field is stored as a different type that
borrows to `T` through `AsRef`, use [`UseFieldRef`](use_field_ref.md). When the field lives several hops
inside a nested context, use [`ChainGetters`](chain_getters.md).

## Under the hood

`UseField<Tag>` is implemented in four places, each forwarding to the context's
[`HasField`](../traits/field-access/has_field.md) implementation for `Tag`.

**The getter component's own provider trait.** For a single-method getter,
[`#[cgp_getter]`](../macros/cgp_getter.md) generates a `UseField` implementation with the field tag
left free, and this is the implementation a direct `UseField<Symbol!("...")>` wiring uses. For the
`HasName` getter above, `cargo cgp expand` shows:

```rust
impl<__Context__, __Tag__> NameGetter<__Context__> for UseField<__Tag__>
where
    __Context__: HasField<__Tag__, Value = String>,
{
    fn name(__context__: &__Context__) -> &str {
        __context__.get_field(::core::marker::PhantomData::<__Tag__>).as_str()
    }
}

impl<__Context__, __Tag__> IsProviderFor<NameGetterComponent, __Context__, ()>
for UseField<__Tag__>
where
    __Context__: HasField<__Tag__, Value = String>,
{}
```

The `&str` return reads a `String` field and appends `.as_str()`, the getter macros' return-type
shorthand. The paired [`IsProviderFor`](../traits/wiring/is_provider_for.md) implementation carries
the same `HasField` bound, so a check names a missing field precisely. A getter with more than one
method gets no `UseField` implementation, since one tag cannot name several fields;
[Common Mistakes](#common-mistakes) shows the error.

**The foundational field getters.** `UseField<Tag>` implements
[`FieldGetter`](../traits/field-access/field_getter.md), which reads the field by reference:

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

Two tags appear for a reason. `OutTag` is the tag the *component* asks under, the getter's own
marker, while `Tag` is the *field* tag the provider was parameterized with. The implementation
ignores `OutTag` and reads `Tag` from the context, which is the decoupling: the component's identity
and the field name are independent. The mutable getter
[`MutFieldGetter`](../traits/field-access/mut_field_getter.md) is implemented the same way, requiring
`Context: HasFieldMut<Tag>` and returning `&mut Value`. These are plain traits rather than
components, so the implementations carry no `IsProviderFor` pair; a getter component reaches them
through [`WithField<Tag>`](with_field.md), whose `WithProvider` wrapper does.

**[`TypeProvider`](../components/has_type.md).** `UseField<Tag>` reports the field's `Value` type as an
[abstract type](/docs/reference/glossary#abstract-type), with a matching `IsProviderFor` implementation,
so the *type* of a field can be wired as a context's abstract type. The built-in
`TypeProviderComponent` takes `UseField<Symbol!("width")>` directly, while a
[`#[cgp_type]`](../macros/cgp_type.md) component takes it as `WithField<Symbol!("width")>`, because
the macro generates `UseType` and `WithProvider` implementations but no `UseField` one.

**The handler family.** `Computer` and `AsyncComputer` are implemented for `UseField<Tag>` by
forwarding the computation to the value stored in the field, which must itself implement the
consumer trait; see [`Computer`](../components/handler/computer.md#usage).

## Common Mistakes

**A getter with more than one method cannot be wired to `UseField`.** `#[cgp_getter]` generates the
`UseField` implementation only for a single-method getter, so wiring a two-method `HasFooBar` getter
to `UseField<Symbol!("foo")>` fails at the check:

```text
error[E0277]: the trait bound `cgp::prelude::UseField<cgp::prelude::Symbol<3, cgp::prelude::Chars<'f', cgp::prelude::Chars<'o', cgp::prelude::Chars<'o', Nil>>>>>: IsProviderFor<FooBarGetterComponent, App>` is not satisfied
```

Wire such a getter to [`UseFields`](use_fields.md) when each method reads a same-named field, or split
it into single-method getters when the field names differ.

## Related constructs

- [`#[cgp_getter]`](../macros/cgp_getter.md) — the macro that generates a `UseField` implementation for
  a getter and makes the field a wiring decision.
- [`UseFields`](use_fields.md) — the sibling that keys every method on its own name, with no tag.
- [`UseFieldRef`](use_field_ref.md) — the foundational getter for a stored type that borrows to a
  different return type through `AsRef`.
- [`ChainGetters`](chain_getters.md) — reaches a field on a nested context by composing getters.
- [`WithField`](with_field.md) — the `WithProvider`-adapted alias that binds the same field.
- [`WithProvider`](with_provider.md) — the adapter behind the `WithField` alias.
- [`UseType`](use_type.md) — the abstract-type analogue for a `#[cgp_type]` component.
- [`HasField`](../traits/field-access/has_field.md) and [`#[derive(HasField)]`](../derives/derive_has_field.md) — the
  consumer-side field access this reads, keyed by [`Symbol!`](../macros/symbol.md) or `Index<N>`.
- [`delegate_components!`](../macros/delegate_components.md) — wires it, and
  [`check_components!`](../macros/check_components.md) verifies the field is present.

The ideas behind it:

- [Impl-side dependencies](/docs/concepts/impl-side-dependencies) — value injection through a
  provider's own requirements, of which reading a field is the most common form.

## Source

- Struct, `WithField` alias, and the `FieldGetter`/`MutFieldGetter`/`TypeProvider` impls:
  [`use_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/impls/use_field.rs)
- The traits it implements: [`cgp-field/src/traits/`](https://github.com/contextgeneric/cgp/tree/main/crates/core/cgp-field/src/traits/)
- The `#[cgp_getter]`-generated `UseField` impl:
  [`cgp_getter/use_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/cgp_getter/use_field.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
