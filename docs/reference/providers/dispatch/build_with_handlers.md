---
title: 'BuildWithHandlers — build a record in steps'
description: 'The builder that starts an empty partial record, runs a list of steps that each fill fields, and finalizes the record once every field is set.'
sidebar_label: 'BuildWithHandlers'
sidebar_position: 10
---

# `BuildWithHandlers`

Build a record from a list of steps, starting from an empty builder and finalizing once every field
is set.

## Overview

`BuildWithHandlers<Output, Handlers>` assembles a record whose fields come from separate providers.
It starts from the empty [partial record](/docs/reference/glossary#partial-record) of `Output`,
passes it through a list of builder steps that each set one field or a group of fields, and turns
the finished builder into the concrete `Output`, on a
[**context**](/docs/reference/glossary#context), the type the implementation runs against. Each
step's provider can read the context, so the context usually holds the configuration the record is
built from. The final conversion exists only for a builder with every field set, so a list that
leaves a field unset does not compile. Like every CGP provider, it carries no runtime value.

## Usage

Import it from `cgp::extra::dispatch`. It takes the output record type and a
[`Product!`](../../macros/product.md) list of steps, each a
[`BuildAndSetField`](build_and_set_field.md) for one field or a
[`BuildAndMerge`](build_and_merge.md) for a sub-record's fields:

```rust
use cgp::extra::dispatch::{BuildAndMerge, BuildAndSetField, BuildWithHandlers};

delegate_components! {
    AppBuilder {
        ComputerComponent:
            BuildWithHandlers<App, Product![
                BuildAndMerge<BuildDatabaseConfig>,
                BuildAndSetField<Symbol!("user_agent"), BuildUserAgent>,
            ]>,
    }
}
```

It ignores the input it is given, so it is called with `()`. `Output` needs the builder traits,
which [`#[derive(CgpData)]`](../../derives/derive_cgp_data.md) or
[`#[derive(BuildField)]`](../../derives/derive_build_field.md) gives it. It implements `Computer`,
`TryComputer`, and `Handler`; the fallible forms need every step to support them and the context to
have an error type.

## Examples

An `AppBuilder` context holds the configuration, and two steps build an `App` from it:

```rust
use cgp::prelude::*;
use cgp::extra::dispatch::{BuildAndMerge, BuildAndSetField, BuildWithHandlers};
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

#[cgp_impl(new BuildUserAgent)]
impl<Code, Input> Computer<Code, Input> {
    type Output = String;

    fn compute(
        &self,
        _code: PhantomData<Code>,
        _input: Input,
        #[implicit] agent_name: &str,
    ) -> String {
        format!("{agent_name}/1.0")
    }
}

delegate_components! {
    AppBuilder {
        ComputerComponent:
            BuildWithHandlers<App, Product![
                BuildAndMerge<BuildDatabaseConfig>,
                BuildAndSetField<Symbol!("user_agent"), BuildUserAgent>,
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

`BuildAndMerge` copies `db_url` and `max_connections` from the built `DatabaseConfig`, and
`BuildAndSetField` sets `user_agent`. Each step's provider ignores its input, the partial record,
and reads the configuration it needs from the context as an
[`#[implicit]`](../../attributes/implicit.md) argument. `AppBuilder` is an
[environmental context](/docs/reference/glossary#environmental-context) that stands for the
configuration an `App` is built from.

## When to use it

**Reach for `BuildWithHandlers` to assemble a record from steps you list**, mixing single fields and
merged sub-records. When every step builds a sub-record,
[`BuildAndMergeOutputs`](build_and_merge_outputs.md) takes the sub-record providers directly and
adds the merge steps. To take an enum apart rather than assemble a record, use a matcher such as
[`MatchWithHandlers`](match_with_handlers.md).

## Under the hood

`BuildWithHandlers<Output, Handlers>` carries the output type and the list in `PhantomData`. Its
`Computer` impl takes the empty builder from [`HasBuilder`](../../traits/builder/has_builder.md),
runs the list over it with [`PipeHandlers`](../handler/pipe_handlers.md), and converts the result
with [`FinalizeBuild`](../../traits/builder/finalize_build.md):

```rust
pub struct BuildWithHandlers<Output, Handlers>(pub PhantomData<(Output, Handlers)>);

#[cgp_provider]
impl<Context, Code, Input, Output, Builder, Handlers, Res> Computer<Context, Code, Input>
    for BuildWithHandlers<Output, Handlers>
where
    Output: HasBuilder<Builder = Builder>,
    PipeHandlers<Handlers>: Computer<Context, Code, Builder, Output = Res>,
    Res: FinalizeBuild<Target = Output>,
{
    type Output = Output;

    fn compute(context: &Context, code: PhantomData<Code>, _input: Input) -> Self::Output {
        PipeHandlers::compute(context, code, Output::builder()).finalize_build()
    }
}
```

Each step changes the builder's type, marking the fields it set as present, so `Res` is the builder
type after the last step. The `TryComputer` and `Handler` impls run the list with `try_compute` or
`handle`, return a step's error through `?`, and finalize the same way.

## Common Mistakes

**A list that leaves a field unset does not compile.** Dropping the `user_agent` step leaves the
final builder with that field absent, and `FinalizeBuild` exists only once every field is present,
so the check reports the builder by its markers, `IsNothing` for the missing `user_agent`:

```text
error[E0277]: the trait bound `__PartialApp<IsPresent, IsPresent, IsNothing>: FinalizeBuild` is not satisfied
```

Add a step for the field, or merge a sub-record that carries it.

## Related constructs

- [`BuildAndSetField`](build_and_set_field.md), [`BuildAndMerge`](build_and_merge.md) — the steps
  its list is built from.
- [`BuildAndMergeOutputs`](build_and_merge_outputs.md) — takes sub-record providers and adds the
  merge steps.
- [`PipeHandlers`](../handler/pipe_handlers.md) — the pipeline it runs the builder through.
- [`HasBuilder`](../../traits/builder/has_builder.md),
  [`FinalizeBuild`](../../traits/builder/finalize_build.md) — the traits that start and finish the
  build.

The ideas behind it:

- [Extensible records](/docs/concepts/extensible-records) — building a record field by field,
  checked at compile time.

## Source

- [`providers/with_handlers/build_with_handlers.rs`](https://github.com/contextgeneric/cgp/blob/main/crates/extra/cgp-dispatch/src/providers/with_handlers/build_with_handlers.rs)

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
