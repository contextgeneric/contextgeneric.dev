---
title: 'BuildAndSetField — set one built field'
description: 'The builder step that computes one field''s value from a provider, which can read the fields already set, and sets it on the partial record.'
sidebar_label: 'BuildAndSetField'
sidebar_position: 8
---

# `BuildAndSetField`

Compute one field's value and set it on a
[partial record](/docs/reference/glossary#partial-record).

## Overview

`BuildAndSetField<Tag, Provider>` is the builder step for a single field. It takes the builder, a
partial record with some fields set, runs `Provider` over a reference to it to compute the value of
the field named `Tag`, and returns the builder with that field set, on a
[**context**](/docs/reference/glossary#context), the type the implementation runs against. Because
`Provider` receives the builder, it can read the fields earlier steps have set as well as anything
on the context. `Provider` defaults to [`UseContext`](../use_context.md). Like every CGP provider,
it carries no runtime value.

## Usage

Import it from `cgp::extra::dispatch`. It takes the field's tag, a
[`Symbol!`](../../macros/symbol.md) naming it, and the provider that computes its value, and is
written as a step in a [`BuildWithHandlers`](build_with_handlers.md) list:

```rust
use cgp::extra::dispatch::{BuildAndSetField, BuildWithHandlers};

delegate_components! {
    PoolBuilder {
        ComputerComponent:
            BuildWithHandlers<PoolConfig, Product![
                BuildAndSetField<Symbol!("max_connections"), BuildMaxConnections>,
                BuildAndSetField<Symbol!("max_idle"), BuildMaxIdle>,
            ]>,
    }
}
```

The provider's output type must equal the field's type. It implements `Computer`, `TryComputer`, and
`Handler`, running the provider as the same member; the fallible forms need the context to have an
error type and return the provider's error.

## Examples

The second step reads the field the first one set:

```rust
use cgp::prelude::*;
use cgp::extra::dispatch::{BuildAndSetField, BuildWithHandlers};
use cgp::extra::handler::CanCompute;

#[derive(Debug, PartialEq, CgpData)]
pub struct PoolConfig {
    pub max_connections: u32,
    pub max_idle: u32,
}

#[derive(HasField)]
pub struct PoolBuilder {
    pub connections: u32,
}

#[cgp_impl(new BuildMaxConnections)]
impl<Code, Input> Computer<Code, Input> {
    type Output = u32;

    fn compute(
        &self,
        _code: PhantomData<Code>,
        _input: Input,
        #[implicit] connections: u32,
    ) -> u32 {
        connections
    }
}

// Reads `max_connections` from the partial record, which the step before it has set.
#[cgp_new_provider]
impl<'a, Context, Code, Builder> Computer<Context, Code, &'a Builder> for BuildMaxIdle
where
    Builder: HasField<Symbol!("max_connections"), Value = u32>,
{
    type Output = u32;

    fn compute(_context: &Context, _code: PhantomData<Code>, builder: &'a Builder) -> u32 {
        builder.get_field(PhantomData) / 2
    }
}

delegate_components! {
    PoolBuilder {
        ComputerComponent:
            BuildWithHandlers<PoolConfig, Product![
                BuildAndSetField<Symbol!("max_connections"), BuildMaxConnections>,
                BuildAndSetField<Symbol!("max_idle"), BuildMaxIdle>,
            ]>,
    }
}

check_components! {
    PoolBuilder {
        ComputerComponent: ((), ()),
    }
}

pub fn demo() {
    let builder = PoolBuilder { connections: 8 };

    assert_eq!(
        builder.compute(PhantomData::<()>, ()),
        PoolConfig {
            max_connections: 8,
            max_idle: 4,
        },
    );
}
```

A partial record implements [`HasField`](../../traits/field-access/has_field.md) for the fields it
has set, so `BuildMaxIdle` bounds on the one it needs, and listing it before the
`max_connections` step would fail that bound. `BuildMaxConnections` reads the context's
`connections` field instead. `PoolBuilder` is an
[environmental context](/docs/reference/glossary#environmental-context) holding the configuration.

## When to use it

**Reach for `BuildAndSetField` when a step computes one named field**, in a list for
[`BuildWithHandlers`](build_with_handlers.md). When a provider already builds a record holding
several of the fields, [`BuildAndMerge`](build_and_merge.md) copies them in one step.

## Under the hood

`BuildAndSetField<Tag, Provider>` carries the tag and provider in `PhantomData`. Its `Computer` impl
runs `Provider` on a reference to the builder, then sets the field with
[`BuildField<Tag>`](../../traits/builder/build_field.md), which returns a builder of a new type with
that field marked present:

```rust
pub struct BuildAndSetField<Tag, Provider = UseContext>(pub PhantomData<(Tag, Provider)>);

#[cgp_provider]
impl<Context, Code, Tag, Value, Provider, Output, Builder> Computer<Context, Code, Builder>
    for BuildAndSetField<Tag, Provider>
where
    Provider: for<'a> Computer<Context, Code, &'a Builder, Output = Value>,
    Builder: BuildField<Tag, Value = Value, Output = Output>,
{
    type Output = Output;

    fn compute(context: &Context, code: PhantomData<Code>, builder: Builder) -> Self::Output {
        let value = Provider::compute(context, code, &builder);
        builder.build_field(PhantomData::<Tag>, value)
    }
}
```

The `for<'a>` bound is why the provider must accept a borrow of the builder at any lifetime. The
`TryComputer` and `Handler` impls run the provider with `try_compute` or `handle` and return its
error through `?`.

## Related constructs

- [`BuildAndMerge`](build_and_merge.md) — the step that copies a sub-record's fields at once.
- [`BuildWithHandlers`](build_with_handlers.md) — runs a list of steps and finalizes.
- [`BuildField`](../../traits/builder/build_field.md) — the trait it drives to set the field.
- [`UseContext`](../use_context.md) — the default provider.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — building a record field by field.

## Source

- [`providers/field_builders/build_and_set_field.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-dispatch/src/providers/field_builders/build_and_set_field.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
