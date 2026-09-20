---
sidebar_label: 'Dispatching'
sidebar_position: 15
---

# Dispatching

Dispatching connects a record's fields or an enum's variants to the providers that process them.
A generic dispatcher can build several record types or handle several enum types while each provider
implements one part of the work. This page explains matching, building, and their compile-time
guarantees, then shows how to derive dispatch from an ordinary trait.

The examples build on [extensible records](./extensible-records.md) and
[extensible variants](./extensible-variants.md). Those pages explain how CGP exposes fields and
variants to generic code; dispatching chooses and runs the computations that use them.

## Two directions, one idea

Dispatchers combine generic data operations with a list of handlers. The data traits describe how
to extract a variant or set a field. A handler supplies the computation to perform on the extracted
payload or to produce the field's value.

Matching and building use these pieces differently:

- **Matching:** Try variants until the current one is found, then run its handler.
- **Building:** Run handlers in sequence, adding their outputs to a [partial record](/docs/reference/glossary#partial-record) until it can be finalized.

Dispatchers implement the same computation interfaces as other
[handler providers](./handlers.md). They can be selected through context wiring, composed with other
computations, or nested to delegate a group of variants to another matcher. The available interfaces
depend on the dispatcher and the providers it contains.

## Matching, and why it is exhaustive

A matcher selects the handler for an enum's current variant. For a `Shape` enum deriving `CgpData`
with `Circle(Circle)` and `Rectangle(Rectangle)` variants, input-based wiring can select both the
matcher and the payload handlers:

```rust
delegate_components! {
    App {
        ComputerComponent: UseInputDelegate<
            new AreaComponents {
                Shape: MatchWithValueHandlers,
                Circle: CircleArea,
                Rectangle: RectangleArea,
            }
        >,
    }
}
```

`UseInputDelegate` selects a provider by the input type. A call to `app.compute(code, shape)` first
reaches `MatchWithValueHandlers`. The matcher extracts the current variant's payload and delegates
it through the same context: a `Circle` reaches `CircleArea`, and a `Rectangle` reaches
`RectangleArea`. The application does not write a `match` over `Shape`.

The matcher tracks unhandled variants through the extractor's remainder type. Each extraction
attempt returns either a payload to handle or a remainder that excludes that variant. The next
attempt receives the narrowed remainder. After every variant has been covered, the remainder is
uninhabited, so finalization can eliminate it without a wildcard or a runtime panic.

A missing handler prevents the matcher from compiling for that enum. An automatically generated
handler list requires a suitable provider for every payload; an explicit list must leave an
uninhabited remainder. Adding a variant therefore requires either an existing generic handler that
covers it or an additional handler.

The attempts are sequenced by the `OkMonadic` pipeline described in
[monadic handlers](./monadic-handlers.md). A successful extraction stops the search; a miss passes
the remainder onward. Here the `Err` branch means “try another variant,” rather than an application
error. Provider selection and exhaustiveness are checked at compile time, while testing the actual
variant and running its handler happen at runtime.

## Building, and why it is complete

A builder dispatcher runs handlers over a partial record and finalizes the result. The partial
record's type tracks which fields are present after each step. `BuildWithHandlers` starts the
builder, passes it through the handler list, and calls `finalize_build` at the end.

Field adapters let a handler produce either one field or a group of fields. `BuildAndSetField`
computes and sets a named field; `BuildAndMerge` computes a source record and merges its fields.
Both pass the current builder by reference to the inner provider, so a later provider can read
fields an earlier step supplied.

Finalization requires every field in the strict builder to be present. Omitting a required field
prevents the pipeline from compiling. Fallible handlers can still return runtime errors while
computing values; completeness guarantees that a successful build supplies every field, not that
construction cannot fail.

This supports the [extensible builder pattern](./extensible-records.md): independent providers
produce parts of a result, and a dispatcher assembles them into a target record. The handler order
must respect any dependencies on fields produced by earlier steps.

## The shortcut for the common case

`#[cgp_auto_dispatch]` forwards an enum's trait methods to its payloads. Use it when each payload
already implements an ordinary trait and the enum should expose that same behavior:

```rust
#[cgp_auto_dispatch]
pub trait CanDescribe {
    fn describe(&self) -> String;
}
```

If `Circle` and `Rectangle` implement `CanDescribe`, a `Shape` enum with the required extensible-data
support gains the trait through a generated [blanket implementation](/docs/reference/glossary#blanket-implementation). A call to `shape.describe()`
forwards to the current payload's implementation without application wiring or a handwritten `match`.
Deriving `CgpData` on `Shape` supplies that data support.

Explicit dispatch combinators provide more control when the context must choose different handlers
or when several variants should be handled as a group. The automatic trait form is a simpler entry
point when forwarding to payload methods is all the operation needs.

## What it costs

Recursive data can create a trait-resolution cycle in directly wired matchers. A payload handler
may require the enum's computation, which requires the matcher, which in turn requires the payload
handler. A context-specific wrapper provider can break that cycle by declaring the enum's provider
implementation and calling the matcher inside its method body. Non-recursive `Shape` wiring does
not need that extra provider.

Missing handlers can produce errors involving partial-variant types or unsatisfied provider bounds.
An incomplete builder similarly reports bounds involving its partial-record state. These diagnostics
can be longer and less direct than an ordinary non-exhaustive `match` or missing struct field.

Dispatching adds type-level work for each field or variant. The compiler resolves the handler chain
and checks its changing types; wide shapes and many instantiations can increase that work. Runtime
variant tests and handler calls still occur, and their optimization depends on the generated code
and compiler. Static wiring alone does not guarantee that dispatch has zero runtime cost.

An ordinary `match` or struct literal is usually clearer when one place owns the whole operation.
Dispatching becomes useful when generic code must work over different shapes or when independently
written providers need to contribute parts of an operation.

## Where to go next

These pages explain the data operations and computation interfaces behind dispatch:

- [Extensible variants](./extensible-variants.md): Extraction, remainders, and exhaustiveness.
- [Extensible records](./extensible-records.md): Partial records and complete construction.
- [Handlers](./handlers.md): The computation interfaces dispatchers implement.
- [Monadic handlers](./monadic-handlers.md): Sequencing attempts and stopping on a result.
- [Dispatch combinators](/docs/reference/providers/dispatch): Matchers, builders, and per-element adapters.
- [`#[cgp_auto_dispatch]`](/docs/reference/macros/cgp_auto_dispatch): Automatic forwarding from an enum
  to its payloads.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
