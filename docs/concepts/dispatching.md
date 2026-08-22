---
sidebar_label: 'Dispatching'
sidebar_position: 15
---

# Dispatching

Routing each field of a record, or the current variant of an enum, to the handler responsible for
it.

This page answers *what actually runs the per-variant and per-field handlers?* It is the layer that
turns [extensible records](./extensible-records.md) and
[extensible variants](./extensible-variants.md) into working code, and it assumes both. It covers the
two directions, the guarantee each carries, and the shortcut for the common case. It closes on the costs,
including the one piece of ceremony that appears once the data is recursive.

## Two directions, one idea

The extensible-data machinery can take an enum apart one variant at a time and can assemble a record one
field at a time. Neither decides *what to do* with each piece. Dispatching is that decision: a
type-level list of handlers, one per element, run as an ordinary
[handler provider](./handlers.md).

It runs in two directions, and they are mirror images.

**Matching** consumes a sum. The value is converted into its extractor, and the handlers are tried in
turn until one matches, so the list stops at the first success. **Building** produces a product. The
builder starts empty, and every handler runs, each setting one field, so the list runs to the end.

Both come out as `Computer` or `Handler` providers, which is the point: a dispatcher goes anywhere a
computation goes. It can be wired to a context, nested inside another dispatcher to handle a group of
variants with a sub-matcher, or chained with other combinators.

## Matching, and why it is exhaustive

The matcher is a chain of attempts, sequenced by the ok monad from
[monadic handlers](./monadic-handlers.md): each step returns success or a remainder, the chain
short-circuits on the first success, and a miss threads the narrowed remainder forward.

Wiring it means naming a handler per payload type, plus an entry routing the enum itself to the matcher:

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

`app.compute(code, shape)` now reaches `CircleArea` or `RectangleArea` according to the variant in hand,
and no code anywhere contains a `match` over `Shape`.

Exhaustiveness survives the indirection, for the reason
[extensible variants](./extensible-variants.md) gives: each failed attempt rules out one variant in the
type, so after the last handler the remainder is uninhabited and is discharged by a `match` with no
arms. Add a variant without a handler and the remainder becomes inhabited again, so the code stops
compiling. The dispatcher proves it covered everything.

## Building, and why it is complete

The mirror image applies to a record. The builder threads a partial record through the handler list,
each one computing and setting a field, with later handlers able to read fields already set. At the end
the record is finalized into the concrete struct.

The guarantee is the mirror image too: finalizing is available only when every field is present, so a
builder missing a handler cannot finalize and does not compile. A matcher proves it covered every
variant; a builder proves it filled every field.

This is the machinery under the [extensible builder](./extensible-records.md): the dispatcher runs
each subsystem's provider and merges the outputs.

## The shortcut for the common case

Much of the time the per-variant logic is an ordinary Rust trait with one implementation per payload
type, and setting up a dispatcher by hand for that is disproportionate.
`#[cgp_auto_dispatch]` generates the wiring instead:

```rust
#[cgp_auto_dispatch]
pub trait CanDescribe {
    fn describe(&self) -> String;
}
```

Implement `CanDescribe` for `Circle` and for `Rectangle`, and any enum whose variants all implement it
gets it too: `shape.describe()` works with no wiring, no combinator named, and no `match`. It is the
form to reach for first, and the rest of this page shows what it does underneath.

## What it costs

**A recursive shape needs an extra hop written by hand.** The table above names the matcher directly,
which works because a `Shape` never contains another `Shape`. When a variant's payload *is* the enum
again, as in an expression language or a tree, the matcher dispatches back through the same component,
and a thin wrapper provider has to sit between the enum's entry and the matcher to break the resolution
cycle. It is ceremony with no conceptual content, and it is the piece of a hand-wired dispatcher most
likely to puzzle a reader.

**A missing handler reports as a shape, not as a name.** The failure is an unsatisfiable bound over a
partial-variant or partial-record type, which is accurate about what is missing and does not say
"`Rectangle` has no handler".

**There is one hop per element.** Each variant tried is a resolution step, and the whole chain is
resolved at compile time, so this costs compile time rather than run time, and a wide enum
dispatched at many types is somewhere that cost shows.

**And a `match` is still usually right.** Dispatching pays when the shape is not known where the logic is
written, or when handlers must be contributed independently. For a closed enum handled in one place, the
language's own construct is clearer and free.

## Where to go next

[Extensible variants](./extensible-variants.md) and [extensible records](./extensible-records.md) are
the two halves this operates on, and both are worth reading first.
[Handlers](./handlers.md) is the interface every dispatcher and every per-element handler speaks, and
[monadic handlers](./monadic-handlers.md) is the chaining the matcher is built from.

For the constructs, [the dispatch combinators](/docs/reference/providers/dispatch_combinators) is the
catalogue of matchers, builders, and per-element adapters, and
[`#[cgp_auto_dispatch]`](/docs/reference/macros/cgp_auto_dispatch) is the shortcut above.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
