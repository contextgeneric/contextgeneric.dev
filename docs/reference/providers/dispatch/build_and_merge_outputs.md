---
title: 'BuildAndMergeOutputs — merge whole records'
description: 'The builder that takes a list of sub-record providers, merges each output into the target record by field name, and finalizes it.'
sidebar_label: 'BuildAndMergeOutputs'
sidebar_position: 11
---

# `BuildAndMergeOutputs`

Build a record from a list of providers that each build part of it, merging every output in by field
name.

## Overview

`BuildAndMergeOutputs<Output, Handlers>` assembles a record from providers that each build a
sub-record: a database configuration, an HTTP configuration, a logger. It wraps each provider in the
[`BuildAndMerge`](build_and_merge.md) step and runs the list through
[`BuildWithHandlers`](build_with_handlers.md), on a
[**context**](/docs/reference/glossary#context), the type the implementation runs against. The
providers know nothing of the target record or of each other; the merge matches their fields to the
target's by name. Like every CGP provider, it carries no runtime value.

## Usage

Import it from `cgp::extra::dispatch`. It takes the output record type and a
[`Product!`](../../macros/product.md) list of sub-record providers:

```rust
use cgp::extra::dispatch::BuildAndMergeOutputs;

delegate_components! {
    AppBuilder {
        ComputerComponent:
            BuildAndMergeOutputs<App, Product![BuildDatabaseConfig, BuildHttpConfig]>,
    }
}
```

Between them, the sub-records must set every field of `Output` once. It answers `ComputerComponent`,
`TryComputerComponent`, and `HandlerComponent`, running each provider as the same member, so a
builder whose providers are handlers is wired to `HandlerComponent`.

## Examples

The same two subsystems as on the [`BuildAndMerge`](build_and_merge.md) page, listed bare:

```rust
use cgp::prelude::*;
use cgp::extra::dispatch::BuildAndMergeOutputs;
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
            BuildAndMergeOutputs<App, Product![BuildDatabaseConfig, BuildHttpConfig]>,
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

The wiring is the `BuildAndMerge` page's list with the two `BuildAndMerge` wrappers left out.
`AppBuilder` is an [environmental context](/docs/reference/glossary#environmental-context) holding
the configuration the providers read.

## When to use it

**Reach for `BuildAndMergeOutputs` when every part of a record comes from a provider that builds a
sub-record**, the extensible builder pattern for assembling an application from its subsystems. When
some steps set single fields, use [`BuildWithHandlers`](build_with_handlers.md) with
[`BuildAndSetField`](build_and_set_field.md) and [`BuildAndMerge`](build_and_merge.md) directly.

## Under the hood

`BuildAndMergeOutputs` is a [`delegate_components!`](../../macros/delegate_components.md) table that
maps the handler components to `BuildWithHandlers` over the list with each provider wrapped in
`BuildAndMerge`:

```rust
delegate_components! {
    <Output, Handlers: MapFields<ToBuildAndMergeHandler>>
    new BuildAndMergeOutputs<Output, Handlers> {
        [
            ComputerComponent,
            ComputerRefComponent,
            TryComputerComponent,
            TryComputerRefComponent,
            HandlerComponent,
            HandlerRefComponent,
        ]:
            BuildWithHandlers<Output, Handlers::Mapped>
    }
}

pub struct ToBuildAndMergeHandler;

impl MapType for ToBuildAndMergeHandler {
    type Map<Handler> = BuildAndMerge<Handler>;
}
```

[`MapFields`](../../traits/type-level/map_fields.md) applies the
[`MapType`](../../traits/type-level/map_type.md) marker `ToBuildAndMergeHandler` to each element of
the list, so `Product![BuildDatabaseConfig, BuildHttpConfig]` becomes
`Product![BuildAndMerge<BuildDatabaseConfig>, BuildAndMerge<BuildHttpConfig>]`.

## Common Mistakes

**The `…Ref` entries do not resolve.** The table routes `ComputerRefComponent`,
`TryComputerRefComponent`, and `HandlerRefComponent`, but `BuildWithHandlers` implements only
`Computer`, `TryComputer`, and `Handler`, so wiring `ComputerRefComponent` to it fails at the check,
and rustc lists the three impls it has:

```text
   | |________________________________________^ `IsProviderFor<cgp::prelude::ComputerComponent, Context, (Code, Input)>`
   | |________________________________________^ `IsProviderFor<cgp::prelude::TryComputerComponent, Context, (Code, Input)>`
   | |________________________________________^ `IsProviderFor<cgp::prelude::HandlerComponent, Context, (Code, Input)>`
```

Wire it to the owned-input components only. The builder ignores its input, so a borrowed form would
add nothing.

## Related constructs

- [`BuildWithHandlers`](build_with_handlers.md) — the builder this runs, for a list of steps written
  directly.
- [`BuildAndMerge`](build_and_merge.md) — the step each provider is wrapped in.
- [`MapFields`](../../traits/type-level/map_fields.md),
  [`MapType`](../../traits/type-level/map_type.md) — the list mapping that adds the wrappers.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — assembling a record from independent
  per-subsystem outputs.

## Source

- [`providers/builders/build_and_merge_outputs.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-dispatch/src/providers/builders/build_and_merge_outputs.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
