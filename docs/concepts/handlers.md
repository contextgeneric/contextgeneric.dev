---
sidebar_label: 'Handlers'
sidebar_position: 11
---

# Handlers

CGP's handler components represent computations through a common input-and-output interface.
Because providers share that interface, a context can select individual computations or compose
them into a pipeline through wiring. The family has separate forms for synchronous and async work,
fallible results, and owned or borrowed inputs, so a simple computation needs only the requirements
its work calls for.

## Computation as something a context decides

An application-specific component gives a name to an operation, such as `send_email` or `load_file`.
A handler gives it a uniform computation interface instead, allowing generic combinators to
connect it to other computations.

In that interface, an input-taking handler receives a context, a `Code` tag, and an `Input`.
The context supplies its dependencies and wiring, while the input carries the value being processed.
The provider chooses the resulting `Output` through an associated type.

The tag matters when several computations accept the same input type: `Code` distinguishes them
so the context can select a provider for each. Passing it as `PhantomData<Code>` carries that type
information without storing a `Code` value. If there is only one computation, `()` can serve as the
tag.

Not every computation needs the same execution or error behavior. Addition, for example, can
return synchronously without an error type or a future. When it needs to join a more general
pipeline, adapters can expose it through the interface that pipeline uses.

## Three axes, and the names that encode them

The main input-taking components vary by execution and error handling. Their consumer traits give
callers the corresponding methods:

| Provider trait | Consumer method | Execution | Result |
| --- | --- | --- | --- |
| [`Computer`](/docs/reference/components/handler/computer) | `compute` | Synchronous | `Output` |
| `AsyncComputer` | `compute_async` | Async | `Output` |
| [`TryComputer`](/docs/reference/components/handler/try_computer) | `try_compute` | Synchronous | `Result<Output, Error>` |
| [`Handler`](/docs/reference/components/handler/handler) | `handle` | Async | `Result<Output, Error>` |

A synchronous method completes its work before it returns, while an async method returns a
future that does the work when polled. The fallible forms add an error channel using the context's
[abstract error type](./modular-error-handling.md). An infallible interface simply lacks that
channel; its implementation may still have side effects or panic.

Input ownership supplies the other distinction. The base components take `Input` by value, while
siblings such as `ComputerRef` and `HandlerRef` take `&Input`. This lets the interface express
whether the computation consumes or borrows its argument.

[`Producer`](/docs/reference/components/handler/producer) supplies a value without an input
argument. It uses the context and tag alone, which suits constants and context-derived values.
Producer adapters can expose that behavior through input-taking interfaces by ignoring the input.

## Write the simplest suitable provider {#write-the-weakest-one-the-rest-come-free}

The separate interfaces need not force a provider to repeat its body. Promotion adapters can
wrap an infallible output in `Ok`, or run synchronous work inside an async method. The computation
keeps its original behavior through those adaptations: in particular, putting blocking work in
a future does not make it nonblocking.

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

These adaptations must respect ownership as well. A provider that consumes an arbitrary input
cannot run on a shared reference to it, so the borrowed interfaces need a compatible implementation.
The direction of execution also matters: placing an async provider in a promotion table cannot
give it a synchronous implementation.

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

To perform that calculation, `PipeHandlers` runs the listed providers from left to right, passing
each output to the next stage. Those types must fit the next stage's input requirements, and the
last stage supplies the pipeline's output. In fallible forms, an error ends the pipeline before
later stages run; in async forms, each stage is awaited before the next begins.

For just two stages, `ComposeHandlers<A, B>` runs `A` and gives its result to `B`. `PipeHandlers`
builds nested uses of that composition from its list. `ReturnInput` provides a stage that leaves
the value unchanged, while the `Promote*` adapters connect compatible computation interfaces.
Their requirements are described in the [handler combinator reference](/docs/reference/providers/handler).

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

If only the input should determine the computation, a generic first segment can match every code:
`@ComputerComponent.<Code> Code.i64`. [Dispatching](./dispatching.md) uses this form for an enum
and its payloads. Because that key also covers `@ComputerComponent.Doubled.i64`, it would overlap
the entries above and cannot appear beside them in the same table. Code tags can also carry more
structure, as the programs in [type-level DSLs](./type-level-dsls.md) show.

## What it costs

Compared with an ordinary function call, even the smallest handler call needs a type tag.
`PhantomData::<()>` fills that role when there is only one computation, but the extra syntax remains.

Larger compositions also require familiarity with the component and adapter family. Execution,
error handling, and ownership explain the names, but a reader must still know which interface each
stage supports. Choosing the simplest form for each computation helps keep those requirements
visible when adapters connect the stages.

Connecting stages through types moves some errors away from the function body. A mismatch can
appear as a trait error involving an associated `Output`, so
[provider checks](./check-traits.md) help isolate the stage whose types do not fit. Once the program
runs, each provider's method body remains available for debugging. The wiring table only selects
those methods, however, so it supplies no executable location for a breakpoint or log statement.

Async handler interfaces do not promise that their returned futures are `Send`. Spawning a task on
an executor that requires `Send` needs an additional guarantee, described in
[Recovering `Send` bounds](./send-bounds.md).

A plain function is simpler when a computation does not need interchangeable stages or context-specific
composition. [`#[cgp_fn]`](/docs/reference/macros/cgp_fn) provides a lighter option when a single
implementation needs values or traits from a context without handler wiring.

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
- [Comparison: Algebraic effects](/docs/comparisons/algebraic-effects): the handler family beside effect handlers, and the continuation CGP never captures.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
