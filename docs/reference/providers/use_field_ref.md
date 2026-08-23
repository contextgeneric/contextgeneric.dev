---
sidebar_label: 'UseFieldRef'
---

# `UseFieldRef`

The foundational field getter that borrows a field through `AsRef`/`AsMut`, for the case where the
stored type differs from the getter's return type.

## Overview

`UseFieldRef<Tag, Value>` reads the field named by `Tag` from the **context** (the type the capability
runs against, which supplies the field) and borrows it through `AsRef` to produce a `&Value`, where the
stored field type implements `AsRef<Value>`. It exists for a getter whose return type is reached
*through* a field rather than being the field's own type: a `&Config` from a stored `Arc<Config>`, for
example, where the field is not a `Config` but can be borrowed as one.

`UseFieldRef` is a foundational [`FieldGetter`](../traits/field_getter.md) rather than a getter
component's own provider, so it is wired through the [`WithProvider`](with_provider.md) adapter using its
`WithFieldRef` alias. That distinguishes it from [`UseField`](use_field.md), which
[`#[cgp_getter]`](../macros/cgp_getter.md) generates a getter-component implementation for directly.

Most borrowed-view getters do not need `UseFieldRef` at all, and this is the key thing to know before
reaching for it. When a getter's return type is one of the shorthands the getter macros recognize,
`&str` or `&[u8]`, the generated [`UseField`](use_field.md) implementation already borrows for you: it
reads the field and calls `as_str()` or `as_ref()`, so a `&str` getter over a `String` field, or a
`&[u8]` getter over a `Vec<u8>` field, is wired with a plain `UseField`. `UseFieldRef` is for the
remaining case, a getter that returns `&T` for some type `T` the field is not stored as but can be
borrowed as. Like every CGP provider, it carries no runtime value.

## Usage

`UseFieldRef` is not in the prelude, and neither is its `WithFieldRef` alias. Import them from
`cgp::core::field::impls`:

```rust
use cgp::core::field::impls::WithFieldRef;
```

`WithFieldRef<Tag, Value>` is [`WithProvider<UseFieldRef<Tag, Value>>`](with_provider.md), and it is the
form you wire, since the bare `UseFieldRef` provides the foundational `FieldGetter` rather than a named
getter's provider trait. It takes the field tag and the borrowed value type, and appears as the value of
a getter component's wiring entry:

```rust
delegate_components! {
    App {
        ConfigGetterComponent: WithFieldRef<Symbol!("config"), Config>,
    }
}
```

`Tag` names the field, as in [`UseField`](use_field.md), and `Value` is the type the getter exposes. The
stored field type must implement `AsRef<Value>` (and, for the mutable getter, `AsMut<Value>`), and the
getter's return type must be `&Value`.

## Examples

A getter returns `&Config` while the context stores the config in a wrapper that borrows as `Config`.
The wrapper implements `AsRef<Config>`, so `WithFieldRef` reads it and borrows through it:

```rust
use cgp::prelude::*;
use cgp::core::field::impls::WithFieldRef; // not in the prelude

pub struct Config {
    pub port: u16,
}

pub struct StoredConfig(pub Config);

impl AsRef<Config> for StoredConfig {
    fn as_ref(&self) -> &Config {
        &self.0
    }
}

#[cgp_getter]
pub trait HasConfig {
    fn config(&self) -> &Config;
}

#[derive(HasField)]
pub struct App {
    pub config: StoredConfig,
}

delegate_components! {
    App {
        ConfigGetterComponent: WithFieldRef<Symbol!("config"), Config>,
    }
}
```

`App` wires `ConfigGetterComponent` to `WithFieldRef<Symbol!("config"), Config>`. The provider reads the
`config` field, a `StoredConfig`, and because `StoredConfig: AsRef<Config>`, returns `&Config` from
`as_ref()`. The getter exposes the borrowed `Config` view while the context owns the `StoredConfig`.

## When to reach for it, and when not

**Reach for `UseFieldRef` when a getter returns `&T` and the context stores a different type that
borrows as `T` through `AsRef`.** The stored type and the exposed type differ, and neither the plain
[`UseField`](use_field.md) nor an [`#[implicit]`](../attributes/implicit.md) argument can bridge them.

Prefer [`UseField`](use_field.md) for the common borrowed-view getters. A `-> &str` getter over a
`String` field and a `-> &[u8]` getter over a `Vec<u8>` field are handled by the generated `UseField`
implementation, which borrows through `as_str()` or `as_ref()` based on the return type, so they need no
`UseFieldRef`. And for a field returned as its own type, an `#[implicit]` argument is simpler still.

## Under the hood

`UseFieldRef<Tag, Value>` implements the provider-side getter
[`FieldGetter`](../traits/field_getter.md) by reading the field at `Tag` and dereferencing it to
`&Value`:

```rust
impl<Context, OutTag, Tag, Value> FieldGetter<Context, OutTag> for UseFieldRef<Tag, Value>
where
    Context: HasField<Tag, Value: AsRef<Value> + 'static>,
{
    type Value = Value;

    fn get_field(context: &Context, _tag: PhantomData<OutTag>) -> &Value {
        context.get_field(PhantomData).as_ref()
    }
}
```

The `where` clause carries the defining constraint: the context's field at `Tag` must implement
`AsRef<Value>`, so the stored type can be borrowed as the exposed type. As in
[`UseField`](use_field.md), `OutTag` is the tag the component asks under and is ignored, while the field
is read at `Tag`. The body reads the field and calls `as_ref()`. The `'static` bound on the field type
lets Rust infer the borrow's lifetime through the `AsRef` call.

`UseFieldRef` also implements the mutable getter [`MutFieldGetter`](../traits/mut_field_getter.md),
requiring the field type to implement both `AsRef<Value>` and `AsMut<Value>` and returning `&mut Value`
through `as_mut()`. Because these are `FieldGetter` implementations rather than a getter component's own
provider trait, the [`WithProvider`](with_provider.md) adapter behind `WithFieldRef` is what turns
`UseFieldRef` into a provider a getter component can be wired to. Unlike [`UseField`](use_field.md), it
does not implement [`TypeProvider`](../components/has_type.md), because its purpose is borrowed field
access rather than abstract-type resolution.

## Related constructs

- [`UseField`](use_field.md) — the getter provider that reads a field directly, and that already handles
  the `&str`/`&[u8]` borrowed shorthands.
- [`WithProvider`](with_provider.md) — the adapter behind the `WithFieldRef` alias, which turns
  `UseFieldRef` into a getter-component provider.
- [`ChainGetters`](chain_getters.md) — another foundational `FieldGetter`, composed for nested contexts.
- [`#[cgp_getter]`](../macros/cgp_getter.md) — the macro that defines the getter this is wired to.
- [`FieldGetter`](../traits/field_getter.md) and [`MutFieldGetter`](../traits/mut_field_getter.md) — the
  provider-side getters it implements, over [`HasField`](../traits/has_field.md).

The ideas behind it:

- [Impl-side dependencies](/docs/concepts/impl-side-dependencies) — value injection through a
  provider's own requirements, of which a getter is one form.

## Source

- Struct, `WithFieldRef` alias, and the `FieldGetter`/`MutFieldGetter` impls:
  [`use_ref.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/impls/use_ref.rs)
- The traits it implements:
  [`has_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/has_field.rs)
  and [`has_field_mut.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/traits/has_field_mut.rs)

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
