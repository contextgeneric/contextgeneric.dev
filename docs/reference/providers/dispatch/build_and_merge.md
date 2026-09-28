---
title: 'BuildAndMerge — merge a built sub-record'
description: 'The builder step that builds a sub-record with a provider and copies every field it shares with the target into the partial record at once.'
sidebar_label: 'BuildAndMerge'
sidebar_position: 9
---

# `BuildAndMerge`

Build a sub-record and copy every field it shares with the target into the builder at once.

## Overview

`BuildAndMerge<Provider>` is the builder step for a group of fields that one provider produces
together. It runs `Provider` over a reference to the builder to build another record, then copies
each of that record's fields into the builder by name, on a
[**context**](/docs/reference/glossary#context), the type the implementation runs against. A
subsystem can therefore contribute its part of a larger record as a struct of its own, with no
knowledge of the record it ends up in. `Provider` defaults to [`UseContext`](../use_context.md).
Like every CGP provider, it carries no runtime value.

## Usage

Import it from `cgp::extra::dispatch`. It takes the provider that builds the sub-record, and is
written as a step in a [`BuildWithHandlers`](build_with_handlers.md) list, beside other merges or
single-field steps:

```rust
use cgp::extra::dispatch::{BuildAndMerge, BuildWithHandlers};

delegate_components! {
    AppBuilder {
        ComputerComponent:
            BuildWithHandlers<App, Product![
                BuildAndMerge<BuildDatabaseConfig>,
                BuildAndMerge<BuildHttpConfig>,
            ]>,
    }
}
```

The sub-record's fields must all be fields of the target, with the same names and types, and still
unset in the builder; the sub-record needs [`#[derive(CgpData)]`](../../derives/derive_cgp_data.md)
so its fields can be read out. It implements `Computer`, `TryComputer`, and `Handler`, running the
provider as the same member; the fallible forms need the context to have an error type and return
the provider's error.

## Examples

Two subsystems each build their part of an `App`:

```rust
use cgp::prelude::*;
use cgp::extra::dispatch::{BuildAndMerge, BuildWithHandlers};
use cgp::extra::handler::CanCompute;

#[derive(Debug, PartialEq, CgpData)]
pub struct App {
    pub db_url: String,
    pub max_connections: u32,
    pub user_agent: String,
}

#[derive(CgpData)]
pub struct DatabaseConfig {
    pub db_url: String,
    pub max_connections: u32,
}

#[derive(HasField)]
pub struct AppBuilder {
    pub db_path: String,
    pub agent_name: String,
}

#[cgp_impl(new BuildDatabaseConfig)]
impl<Code, Input> Computer<Code, Input> {
    type Output = DatabaseConfig;

    fn compute(
        &self,
        _code: PhantomData<Code>,
        _input: Input,
        #[implicit] db_path: &str,
    ) -> DatabaseConfig {
        DatabaseConfig {
            db_url: format!("sqlite://{db_path}"),
            max_connections: 4,
        }
    }
}

#[derive(CgpData)]
pub struct HttpConfig {
    pub user_agent: String,
}

#[cgp_impl(new BuildHttpConfig)]
impl<Code, Input> Computer<Code, Input> {
    type Output = HttpConfig;

    fn compute(
        &self,
        _code: PhantomData<Code>,
        _input: Input,
        #[implicit] agent_name: &str,
    ) -> HttpConfig {
        HttpConfig {
            user_agent: format!("{agent_name}/1.0"),
        }
    }
}

delegate_components! {
    AppBuilder {
        ComputerComponent:
            BuildWithHandlers<App, Product![
                BuildAndMerge<BuildDatabaseConfig>,
                BuildAndMerge<BuildHttpConfig>,
            ]>,
    }
}

check_components! {
    AppBuilder {
        ComputerComponent: ((), ()),
    }
}

pub fn demo() {
    let builder = AppBuilder {
        db_path: "app.db".to_owned(),
        agent_name: "reader".to_owned(),
    };

    assert_eq!(
        builder.compute(PhantomData::<()>, ()),
        App {
            db_url: "sqlite://app.db".to_owned(),
            max_connections: 4,
            user_agent: "reader/1.0".to_owned(),
        },
    );
}
```

`DatabaseConfig` fills two of `App`'s fields and `HttpConfig` the third, each matched by name.
`AppBuilder` is an [environmental context](/docs/reference/glossary#environmental-context) whose
fields the two providers read as [`#[implicit]`](../../attributes/implicit.md) arguments.

## When to use it

**Reach for `BuildAndMerge` when a provider builds several of the target's fields as a struct of its
own.** For a single field, [`BuildAndSetField`](build_and_set_field.md) needs no sub-record type.
When every step is a merge, as here,
[`BuildAndMergeOutputs`](build_and_merge_outputs.md) takes the providers directly and adds the
`BuildAndMerge` to each.

## Under the hood

`BuildAndMerge<Provider>` carries the provider in `PhantomData`. Its `Computer` impl runs `Provider`
on a reference to the builder and copies the result's fields in with
[`CanBuildFrom`](../../traits/casting/can_build_from.md):

```rust
pub struct BuildAndMerge<Provider = UseContext>(pub PhantomData<Provider>);

#[cgp_provider]
impl<Context, Code, Builder, Provider, Output, Res> Computer<Context, Code, Builder>
    for BuildAndMerge<Provider>
where
    Provider: for<'a> Computer<Context, Code, &'a Builder, Output = Res>,
    Builder: CanBuildFrom<Res, Output = Output>,
{
    type Output = Output;

    fn compute(context: &Context, code: PhantomData<Code>, builder: Builder) -> Self::Output {
        let output = Provider::compute(context, code, &builder);
        builder.build_from(output)
    }
}
```

`build_from` walks the sub-record's field list and sets each field on the builder, so the returned
builder has those fields marked present. The `TryComputer` and `Handler` impls run the provider with
`try_compute` or `handle` and return its error through `?`.

## Related constructs

- [`BuildAndSetField`](build_and_set_field.md) — the step for one field.
- [`BuildWithHandlers`](build_with_handlers.md) — runs a list of steps and finalizes.
- [`BuildAndMergeOutputs`](build_and_merge_outputs.md) — adds this step to each provider in a list.
- [`CanBuildFrom`](../../traits/casting/can_build_from.md) — the trait that copies the shared
  fields.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — assembling a record from independent
  parts.

## Source

- [`providers/field_builders/build_and_merge.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-dispatch/src/providers/field_builders/build_and_merge.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
