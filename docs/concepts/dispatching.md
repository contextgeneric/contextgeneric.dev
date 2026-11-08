---
title: 'Route fields and variants to their handlers'
sidebar_label: 'Dispatching'
sidebar_position: 15
description: 'How a dispatcher routes record fields and enum variants to the providers that process them, and how the compiler checks coverage and completeness.'
---

# Dispatching

[Context-Generic Programming](/docs/) (CGP) extends Rust with pluggable trait implementations that
a dispatcher can route data to. Dispatching connects a record's fields or an enum's variants to the
providers that process them. By separating that connection from the individual computations, a
generic dispatcher can build several record types or handle several enum types while each provider
implements one part of the work. The compiler checks that the pieces fit together and cover the
required fields or variants.

[Extensible records](./extensible-records.md) and [extensible variants](./extensible-variants.md)
expose data to generic code through field and variant traits. Dispatchers use those traits to run
the computations that build a record or handle the current variant.

## Two directions, one idea

The data traits used by a dispatcher describe how to extract a variant or set a field. To do
useful work with those operations, the dispatcher also needs handlers: one can process an extracted
payload, while another can produce a value for a field.

These operations lead to two forms of dispatch. A matcher tries variants until it finds the
current payload, then runs the corresponding handler. A builder instead runs its handlers in
sequence, adding their results to a [partial record](/docs/reference/glossary#partial-record)
until the record can be finalized.

Because dispatchers implement the same interfaces as other [handler providers](./handlers.md),
they can participate in ordinary context wiring and computation pipelines. A matcher can even
delegate a group of variants to another matcher. The interfaces available to the resulting
composition depend on both the dispatcher and the providers it contains.

## Matching, and why it is exhaustive

A matcher selects the handler for an enum's current variant. For a `Shape` enum deriving `CgpData`
with `Circle(Circle)` and `Rectangle(Rectangle)` variants, wiring keyed on the input type can select
both the matcher and the payload handlers:

```rust
delegate_components! {
    App {
        open ComputerComponent;

        @ComputerComponent.<Code> Code.Shape: MatchWithValueHandlers,
        @ComputerComponent.<Code> Code.Circle: CircleArea,
        @ComputerComponent.<Code> Code.Rectangle: RectangleArea,
    }
}
```

The [`open` statement](/docs/reference/macros/delegate_components#choosing-a-provider-per-type-the-open-statement)
keys each entry by one path segment per type parameter of `CanCompute<Code, Input>`. The first
segment, `<Code> Code`, matches any code, so the second segment selects a provider by the input type
alone. A call to `app.compute(code, shape)` first reaches `MatchWithValueHandlers`. The matcher
extracts the current variant's payload and delegates it through the same context: a `Circle` reaches
`CircleArea`, and a `Rectangle` reaches `RectangleArea`. The application does not write a `match`
over `Shape`.

To check coverage, the matcher tracks unhandled variants through the extractor's remainder type.
An extraction attempt either finds a payload to handle or passes on a remainder that excludes that
variant. Each miss therefore leaves fewer possibilities for the next attempt. Once every variant
has been covered, the remainder is uninhabited, so finalization can eliminate it without a wildcard
or a runtime panic.

If a variant remains uncovered, the matcher cannot compile for that enum. With an automatically
generated handler list, every payload must have a suitable provider; with an explicit list, the
extraction steps must leave an uninhabited remainder. Adding a variant therefore needs either a
new handler or an existing generic handler that already covers it.

This sequence of attempts uses the `OkMonadic` pipeline from
[monadic handlers](./monadic-handlers.md). A successful extraction ends the search, while a miss
passes its remainder to the next attempt. In this use of `Result`, `Err` means “try another variant”
rather than an application failure. The compiler checks provider selection and exhaustiveness;
the actual variant tests and handler calls happen when the program runs.

## Building, and why it is complete

Building follows the other direction: handlers supply values until a partial record is complete.
`BuildWithHandlers` starts the builder, passes it through the handler list, and calls `finalize_build`.
After each step, the partial record's type describes the fields supplied so far.

Each step can contribute one field or several. `BuildAndSetField` runs an inner provider and sets
a named field from its output, while `BuildAndMerge` merges the fields of a returned record. Both
lend the current builder to that provider, allowing it to read fields supplied by earlier steps.

As each handler adds fields, the partial record's type records their presence. Finalization
requires every field in the strict builder to be present, so omitting a required field prevents the
pipeline from compiling. Computing those values can still fail at runtime if the handlers are
fallible; the guarantee is that a successful build supplies every field.

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

Once `Circle` and `Rectangle` implement `CanDescribe`, the generated
[blanket implementation](/docs/reference/glossary#blanket-implementation) can forward the same
operation from `Shape`. Deriving `CgpData` supplies the enum support it needs, so
`shape.describe()` reaches the current payload's implementation without a handwritten `match` or
application wiring.

Explicit dispatch combinators provide more control when the context must choose different handlers
or when several variants should be handled as a group. The automatic trait form is a simpler entry
point when forwarding to payload methods is all the operation needs.

## What it costs

Recursive data can create a trait-resolution cycle in directly wired matchers. A payload handler
may require the enum's computation, which requires the matcher, which in turn requires the payload
handler. A context-specific wrapper provider can break that cycle by declaring the enum's provider
implementation and calling the matcher inside its method body. Non-recursive `Shape` wiring does
not need that extra provider.

When these type-level guarantees fail, the errors can be less direct than an ordinary missing
struct field or non-exhaustive `match`. A missing handler may appear as an unsatisfied provider
bound or a partial-variant type. An incomplete build can similarly report the bounds on its
partial-record state.

The compiler also has to resolve the handler chain and its changing types for each field or
variant. Wide [shapes](/docs/reference/glossary#shape) and many instantiations can increase that work.
At runtime, variant tests and handler calls still execute, with optimization depending on the
code generated and the compiler. Static wiring removes provider lookup, without promising that
the dispatch operation itself is free.

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
