---
title: 'WithFieldRef — borrow a field through AsRef'
description: 'The alias WithProvider<UseFieldRef<Tag, Value>>: a getter returning &Value reads a field whose stored type implements AsRef<Value>.'
sidebar_label: 'WithFieldRef'
sidebar_position: 6.4
---

# `WithFieldRef`

Wire a getter component to a field borrowed through `AsRef`, through the `WithProvider` adapter.

## Overview

`WithFieldRef<Tag, Value>` is the alias `WithProvider<UseFieldRef<Tag, Value>>`. It implements a getter
component by reading the field named by `Tag` from the [**context**](/docs/reference/glossary#context), the type the method runs on, and borrowing it through `AsRef` to produce a `&Value`, where the stored field type implements
`AsRef<Value>`. It adapts the foundational [`UseFieldRef`](use_field_ref.md) getter through the
[`WithProvider`](with_provider.md) layer. Like every CGP provider, it carries no runtime value.

Unlike [`WithField`](with_field.md), this alias is not an alternative to a directly-wireable provider:
the bare [`UseFieldRef`](use_field_ref.md) supplies only the foundational
[`FieldGetter`](../traits/field-access/field_getter.md), so `WithFieldRef` is the form you wire.

## Usage

`WithFieldRef` is not in the prelude. Import it from `cgp::core::field::impls`. It takes the field tag
and the borrowed value type, and appears as the value of a getter component's wiring entry:

```rust
use cgp::core::field::impls::WithFieldRef;

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
use cgp::core::field::impls::WithFieldRef;
use cgp::prelude::*;

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

check_components! {
    App {
        ConfigGetterComponent,
    }
}

pub fn demo() {
    let app = App { config: StoredConfig(Config { port: 8080 }) };
    assert_eq!(app.config().port, 8080);
}
```

`App` wires `ConfigGetterComponent` to `WithFieldRef<Symbol!("config"), Config>`. The provider reads the
`config` field, a `StoredConfig`, and because `StoredConfig: AsRef<Config>`, returns `&Config` from
`as_ref()`. The getter exposes the borrowed `Config` view while the context owns the `StoredConfig`.

## When to use it

**Reach for `WithFieldRef` when a getter returns `&T` and the context stores a different type that
borrows as `T` through `AsRef`.** The stored type and the exposed type differ, and neither the plain
[`WithField`](with_field.md) nor an [`#[implicit]`](../attributes/implicit.md) argument can bridge them.

The common borrowed-view getters need no `WithFieldRef`: a `-> &str` getter over a `String` field, a
`-> &[T]` getter over a field implementing `AsRef<[T]>` such as a `Vec<u8>`, and an `Option<&T>` getter
over an `Option<T>` field are all served by the plain [`UseField`](use_field.md) or
[`WithField`](with_field.md) through the getter macros' return-type shorthands. For a field returned
as its own type, an `#[implicit]` argument is simpler still.

## Under the hood

`WithFieldRef<Tag, Value>` is a type alias:

```rust
pub type WithFieldRef<Tag, Value> = WithProvider<UseFieldRef<Tag, Value>>;
```

The [`WithProvider`](with_provider.md) adapter forwards a getter's provider-trait method to the inner
provider's foundational method, and [`UseFieldRef<Tag, Value>`](use_field_ref.md) is the foundational
[`FieldGetter`](../traits/field-access/field_getter.md) that reads the field at `Tag` and borrows it through `AsRef`.
The `UseFieldRef` mechanism, including its mutable form, is documented on the
[`UseFieldRef`](use_field_ref.md) page, and the generated `WithProvider` impl on the
[`WithProvider`](with_provider.md) page.

## Related constructs

- [`WithProvider`](with_provider.md) — the adapter this alias specializes.
- [`UseFieldRef`](use_field_ref.md) — the foundational getter it wraps, where the `AsRef` mechanism is
  documented.
- [`WithField`](with_field.md) — the sibling alias for a field read directly, without the `AsRef` step.
- [`#[cgp_getter]`](../macros/cgp_getter.md) — defines the getter component this is wired to.
- [`FieldGetter`](../traits/field-access/field_getter.md) and [`MutFieldGetter`](../traits/field-access/mut_field_getter.md) — the
  provider-side getters the inner provider implements, over [`HasField`](../traits/field-access/has_field.md).

The ideas behind it:

- [Impl-side dependencies](/docs/concepts/impl-side-dependencies) — value injection through a
  provider's own requirements, of which a getter is one form.

## Source

- The alias:
  [`use_ref.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/core/cgp-field/src/impls/use_ref.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
