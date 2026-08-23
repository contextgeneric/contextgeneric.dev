---
sidebar_label: 'UseFields'
sidebar_position: 4
---

# `UseFields`

Implement a getter component by reading each method's value from the context field named after the
method.

## Overview

`UseFields` is the provider form of the convention "the method `name` reads the field `name`." A getter
component defined with [`#[cgp_getter]`](../macros/cgp_getter.md) describes one or more values the
**context** can supply, and the most common arrangement is that each value lives in a same-named field.
The context is the type the capability runs against, and it supplies those values as its own fields.
`UseFields` is the provider that realizes the arrangement: wiring a getter to `UseFields` makes every
method read the context field whose name equals the method name, looked up through
[`HasField`](../traits/field-access/has_field.md) keyed by a [`Symbol!`](../macros/symbol.md).

This is the provider analogue of the blanket implementation that
[`#[cgp_auto_getter]`](../macros/cgp_auto_getter.md) emits. `#[cgp_auto_getter]` produces a single
blanket implementation that fires automatically for any context whose field names match the method
names, with nothing to wire. `UseFields` packages the same field-by-method-name behavior as a provider
that a [`#[cgp_getter]`](../macros/cgp_getter.md) component can be wired to, so a getter that
participates in CGP wiring can still opt into the auto-getter convention when its fields line up with
its methods.

`UseFields` is distinct from its sibling [`UseField`](use_field.md), and the singular-versus-plural
naming marks the difference. `UseField<Tag>` keys on a tag the wiring chooses, letting one method read
a field of any name; `UseFields` takes no parameter and keys every method on its own name. Reach for
`UseField` when the field name must differ from the method name, and for `UseFields` when the
convention holds. Like every CGP provider, `UseFields` carries no runtime value: it is a marker named
in wiring.

## Usage

`UseFields` is in the prelude, so `use cgp::prelude::*;` is enough. It takes no type parameter and
appears as the value of a getter component's wiring entry:

```rust
delegate_components! {
    App {
        FooGetterComponent: UseFields,
    }
}
```

Every method of the getter reads the context field whose name equals the method name. The context must
have a `HasField` implementation for each of those names, which
[`#[derive(HasField)]`](../derives/derive_has_field.md) supplies. The return-type shorthands the getter
macros support apply here too, so a `&str` return reads a `String` field and appends `.as_str()`.

## Examples

A context whose field name matches the getter method can be wired to `UseFields` to get the auto-getter
convention inside a `#[cgp_getter]` component. The method `foo` and the field `foo` share a name:

```rust
use cgp::prelude::*;

#[cgp_getter]
pub trait HasFoo {
    fn foo(&self) -> &str;
}

#[derive(HasField)]
pub struct App {
    pub foo: String,
}

delegate_components! {
    App {
        FooGetterComponent: UseFields,
    }
}

fn describe(app: &App) -> &str {
    app.foo() // reads the `foo` field
}
```

Because `App` wires `FooGetterComponent` to `UseFields`, the getter reads `App`'s `foo` field, the
field whose name equals the method `foo`. If the value were stored under a differently named field,
this wiring would not apply, and the context would wire [`UseField<Symbol!("...")>`](use_field.md) with
the actual field name instead.

## When to use it

**Reach for `UseFields` when a `#[cgp_getter]` component's methods each read a same-named field.** It is
the wired counterpart of the [`#[cgp_auto_getter]`](../macros/cgp_auto_getter.md) convention, for the
case where the getter is a full component rather than a blanket impl.

Prefer [`#[cgp_auto_getter]`](../macros/cgp_auto_getter.md) when the getter does not need to be a
wireable component, and prefer an [`#[implicit]`](../attributes/implicit.md) argument when a provider
simply needs a value from its own context, which is the common case and needs no getter trait at all.
Reach for [`UseField<Tag>`](use_field.md) instead when the field name must differ from the method name.

## Under the hood

[`#[cgp_getter]`](../macros/cgp_getter.md) generates a `UseFields` implementation of the getter's
provider trait, reading each method's value from the field whose name matches the method, keyed by a
`Symbol!`. For a single-method getter such as

```rust
#[cgp_getter]
pub trait HasFoo {
    fn foo(&self) -> &str;
}
```

the macro emits this `UseFields` implementation for the generated `FooGetter` provider trait (shown
with the macro's real placeholder identifiers, and with `Symbol!("foo")` in sugared form):

```rust
impl<__Context__> FooGetter<__Context__> for UseFields
where
    __Context__: HasField<Symbol!("foo"), Value = String>,
{
    fn foo(__context__: &__Context__) -> &str {
        __context__.get_field(PhantomData::<Symbol!("foo")>).as_str()
    }
}
```

Each method becomes a `HasField` bound keyed on the method name as a `Symbol!`, and the body reads that
field. The `&str` return makes the field `Value` a `String` and appends `.as_str()`, the same shorthand
[`#[cgp_auto_getter]`](../macros/cgp_auto_getter.md) uses. When a getter has several methods, the
implementation carries one `HasField` bound and one body per method, each keyed by its own name. The
implementation is paired with a matching [`IsProviderFor`](../traits/wiring/is_provider_for.md), so a check
reports a missing field precisely.

This is one of three provider implementations `#[cgp_getter]` generates for a getter component. The
other two are [`UseField`](use_field.md), for a wiring-chosen field name, and
[`WithProvider`](with_provider.md), for adapting a foundational field getter. A context picks among
them at wiring time.

## Related constructs

- [`#[cgp_getter]`](../macros/cgp_getter.md) — the macro that generates this implementation.
- [`#[cgp_auto_getter]`](../macros/cgp_auto_getter.md) — the blanket-impl form of the same convention,
  for a getter that need not be wired.
- [`UseField`](use_field.md) — the sibling that keys on a wiring-chosen tag instead of the method name.
- [`WithProvider`](with_provider.md) — the third getter provider `#[cgp_getter]` emits.
- [`HasField`](../traits/field-access/has_field.md) and [`#[derive(HasField)]`](../derives/derive_has_field.md) — the
  field access it reads, keyed by [`Symbol!`](../macros/symbol.md) on the method name.

The ideas behind it:

- [Impl-side dependencies](/docs/concepts/impl-side-dependencies) — value injection through a
  provider's own requirements, of which a getter is one form.

## Source

- Struct:
  [`use_fields.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-component/src/providers/use_fields.rs)
- The `#[cgp_getter]`-generated `UseFields` impl:
  [`cgp_getter/to_use_fields_impl.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/macros/cgp-macro-core/src/types/cgp_getter/to_use_fields_impl.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
