---
sidebar_label: 'Handlers'
sidebar_position: 11
---

# Handlers

CGP's handler components let a context select and compose computations through wiring. Separate
interfaces describe whether a computation is synchronous or async, returns a fallible result, and
consumes or borrows its input. This page explains those choices, the adapters between them, and how
providers form a pipeline.

## Computation as something a context decides

Handler components provide a common interface for computations that a context can choose.
An application-specific component might expose `send_email` or `load_file`; handlers expose a
uniform input-and-output interface that generic combinators can connect.

An input-taking handler receives a context, a `Code` tag, and an `Input`, then produces an `Output`.
The context supplies dependencies and wiring. The tag identifies the computation, and the input
contains the value being processed. The provider selects `Output` through an associated type.

The `Code` tag lets one context select several computations for the same input type. It is passed
as `PhantomData<Code>`, which carries type information without storing a `Code` value. A program
with only one computation can use `()` as its tag.

Different interfaces keep unnecessary requirements out of simple computations. An addition provider
can return its output synchronously without requiring an error type or returning a future. Adapters
can expose that provider through a more general interface when composition requires it.

## Three axes, and the names that encode them

The main input-taking components vary by execution and error handling. Their consumer traits give
callers the corresponding methods:

| Provider trait | Consumer method | Execution | Result |
| --- | --- | --- | --- |
| [`Computer`](/docs/reference/components/handler/computer) | `compute` | Synchronous | `Output` |
| `AsyncComputer` | `compute_async` | Async | `Output` |
| [`TryComputer`](/docs/reference/components/handler/try_computer) | `try_compute` | Synchronous | `Result<Output, Error>` |
| [`Handler`](/docs/reference/components/handler/handler) | `handle` | Async | `Result<Output, Error>` |

Synchronous methods finish their work before returning; async methods return futures that perform
work when polled. Fallible interfaces use the context's
[abstract error type](./modular-error-handling.md). “Infallible” describes the interface's lack of an
error channel; it does not guarantee that the implementation cannot panic or perform side effects.

Input ownership supplies the other distinction. The base components take `Input` by value, while
siblings such as `ComputerRef` and `HandlerRef` take `&Input`. This lets the interface express
whether the computation consumes or borrows its argument.

[`Producer`](/docs/reference/components/handler/producer) supplies a value without an input
argument. It uses the context and tag alone, which suits constants and context-derived values.
Producer adapters can expose that behavior through input-taking interfaces by ignoring the input.

## Write the simplest suitable provider {#write-the-weakest-one-the-rest-come-free}

Promotion adapters let a provider support another computation interface without duplicating its
body. An infallible result can be wrapped in `Ok`, and synchronous work can be performed inside an
async method. These adaptations preserve the computation; wrapping blocking work in a future does
not make the work nonblocking.

`#[cgp_computer]` generates a provider and its promotion wiring from a function:

```rust
#[cgp_computer]
fn add(a: u64, b: u64) -> u64 {
    a + b
}
```

The macro preserves `add` and creates an `Add` provider. Its input is the tuple `(u64, u64)`, so a
direct call takes the form `Add::compute(&app, PhantomData::<()>, (1, 2))`. Promotion also exposes
`try_compute`, `compute_async`, and `handle`; the fallible forms require the context's error-type
support even though this function does not return an error.

A `Result`-returning function can also expose its error through the fallible interfaces:

```rust
#[cgp_computer]
fn checked_add(a: u64, b: u64) -> Result<u64, String> {
    a.checked_add(b).ok_or_else(|| "overflow".to_owned())
}
```

`CheckedAdd::compute` returns `Result<u64, String>` as its output value. The promoted
`try_compute` and `handle` forms treat that result as success or failure and require a context whose
error type matches `String`. Promotion does not automatically convert an arbitrary source error into
another error type.

Borrowed interfaces require a compatible borrowed-input implementation. Promotion does not let a
provider consume an arbitrary value through a shared reference. Likewise, an async provider does
not gain a synchronous implementation merely by being placed in a promotion table.

Choose the simplest interface that expresses the computation, then use the adapters supported by
its signature. The function macros generate suitable promotion wiring; a manually implemented
provider needs explicit adapters when another interface is required.

## A pipeline is a wiring entry

A handler pipeline is a provider that passes each stage's output to the next stage. Because the
stages share the handler interface, the context can select the whole pipeline through one entry:

```rust
delegate_components! {
    App {
        ComputerComponent: PipeHandlers<
            Product![
                Multiply<Symbol!("foo")>,
                Add<Symbol!("bar")>,
                Multiply<Symbol!("baz")>,
            ]
        >,
    }
}
```

Here `Multiply<Tag>` and `Add<Tag>` are field-reading providers: each reads the context field
identified by `Tag` and combines it with the input. This `Add<Tag>` is separate from the
function-generated `Add` in the previous example. The pipeline computes `((5 * foo) + bar) * baz`
when given input `5`.

`PipeHandlers` runs the listed providers from left to right. Each stage's output type must satisfy
the next stage's input requirements, and the final stage determines the pipeline's output. The
fallible forms stop on the first error; async forms await each stage in sequence.

`ComposeHandlers<A, B>` is the two-stage form: run `A`, then pass its result to `B`. `PipeHandlers`
constructs nested compositions from its provider list. `ReturnInput` passes its input through
unchanged, and the `Promote*` adapters connect compatible interfaces. The
[handler combinator reference](/docs/reference/providers/handler) describes their bounds.

## Many computations on one context

Different `Code` tags let the same context select different computations for the same input type:

```rust
delegate_components! {
    App {
        open ComputerComponent;

        @ComputerComponent.Doubled.i64: ComputeDoubled,
        @ComputerComponent.Negated.i64: ComputeNegated,
    }
}
```

`App.compute(PhantomData::<Doubled>, 21i64)` selects `ComputeDoubled`, while the `Negated` tag selects
`ComputeNegated`. This table names a code tag followed by the input type for each entry, so the
compiler can resolve the requested pair.

Input-driven dispatch can instead use `UseInputDelegate` with a table keyed by input type.
[Dispatching](./dispatching.md) uses that form to route an enum and its payloads to different
providers. The tag can also describe a larger program, as in [type-level DSLs](./type-level-dsls.md).

## What it costs

A handler call needs a type tag even when the application has only one computation.
`PhantomData::<()>` supplies it, but adds syntax compared with an ordinary function call.

The family introduces several component and adapter names. The execution, error, and ownership
choices explain their relationships, but reading a pipeline still requires knowing which
interfaces each stage supports. Start with the form the computation needs and consult the adapter
bounds when connecting it to another form.

Type-level composition makes some failures harder to locate. A mismatched stage can produce a trait
error involving an associated `Output` type rather than a line in a function body.
[Provider checks](./check-traits.md) help isolate the incompatible stage. Runtime debugging remains
possible inside each provider's method body, although the wiring table itself is not executable
code where a breakpoint or log statement can be placed.

Async handler interfaces do not promise that their returned futures are `Send`. Spawning a task on
an executor that requires `Send` needs an additional guarantee, described in
[Recovering `Send` bounds](./send-bounds.md).

A plain function is simpler when a computation does not need interchangeable stages or context-specific
composition. [`#[cgp_fn]`](/docs/reference/macros/cgp_fn) provides a lighter option when a single
implementation needs values or capabilities from a context without handler wiring.

## Where to go next

These pages explain composition and specify the handler interfaces:

- [Monadic handlers](./monadic-handlers.md): Pipelines whose results decide which branch continues.
- [Dispatching](./dispatching.md): Handlers applied to record construction and enum variants.
- [Type-level DSLs](./type-level-dsls.md): Tags that represent programs.
- [Higher-order providers](./higher-order-providers.md): The provider composition used by combinators.
- [`Computer`](/docs/reference/components/handler/computer),
  [`TryComputer`](/docs/reference/components/handler/try_computer),
  [`Handler`](/docs/reference/components/handler/handler), and
  [`Producer`](/docs/reference/components/handler/producer): Component signatures and requirements.
- [`#[cgp_computer]`](/docs/reference/macros/cgp_computer) and
  [`#[cgp_producer]`](/docs/reference/macros/cgp_producer): Generating providers from functions.
- [Handler combinators](/docs/reference/providers/handler): Composition and promotion adapters.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
