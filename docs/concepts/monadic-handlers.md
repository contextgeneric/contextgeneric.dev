---
sidebar_label: 'Monadic handlers'
sidebar_position: 12
---

# Monadic handlers

Chaining handlers into a pipeline that stops at the first step producing a result the chain should
return.

This page answers *how does a pipeline handle a step that should end it?* It is short, and it assumes
[handlers](./handlers.md). It shows where plain composition stops working, what a monad supplies that
fixes it, and the three CGP ships. It closes on when this is more machinery than a `?` operator.

## Where plain composition stops

Chaining handlers works by feeding each output into the next input, which is right exactly when every
step always wants to continue.

The moment a step can produce a value meaning *stop, this is the answer*, it is wrong. Take a
computation that can overflow:

```rust
#[cgp_computer]
pub fn increment(value: u8) -> Result<u8, &'static str> {
    value.checked_add(1).ok_or("overflow")
}
```

Chaining two of these plainly does not even compile: the first produces a `Result<u8, _>` and the second
wants a `u8`. And forcing the types to line up would be worse than the type error, because the later
steps would run on a value they were never meant to see.

`Result` is the familiar case, and the shape is more general than `Result`. Any output type carrying two
possibilities — one to carry on with, one to return immediately — has the same problem.

## What a monad supplies

A **monad**, in this setting, is a small answer to three questions about an output type: which case
threads forward into the next step, which case short-circuits out as the final result, and how a plain
value is lifted back into the type. Nothing more abstract than that is needed to read this page.

Given those answers, a list of handlers can be composed so each step runs only on the "continue" branch
of the one before, and a "stop" value flows untouched to the end:

```rust
PipeMonadic::<ErrMonadic, Product![Increment, Increment, Increment]>
```

Starting from `1` this produces `Ok(4)`. Starting from `253` the third step overflows, so the pipeline's
output is `Err("overflow")` and the remaining steps do not run. That is the `?` operator, expressed as a
type.

## The three CGP ships

**The err monad** continues on `Ok` and short-circuits on `Err` — the `?`-style early return above,
where the first failure wins. It is the one most pipelines want.

**The ok monad** is its mirror: it continues on `Err` and stops on `Ok`. That sounds backwards until you
want a chain of attempts where the first *success* ends it — a lookup tried against several sources, a
parser trying alternatives.

**The identity monad** never short-circuits and threads every value forward, which recovers plain
composition. It exists so that "no short-circuiting" is a choice you make rather than a different
combinator you reach for, and it is what a `Result`-producing chain almost never wants.

They also stack, so a pipeline over a nested `Result<Result<T, E>, F>` can short-circuit on the outer
error while threading the inner result.

## It is still just a provider

The pipeline that comes out is an ordinary provider for the [handler family](./handlers.md), so it goes
in a table like anything else and nothing downstream knows a monad was involved:

```rust
delegate_components! {
    App {
        ComputerComponent: PipeMonadic<ErrMonadic, Product![Increment, Increment]>,
    }
}
```

The whole construction lives in types — the monad is a zero-sized marker, the handler list is a
type-level list, and the pipeline carries no runtime value — so the only branching at run time is the
branching the logic actually asked for.

## What it costs

**It is a word that scares people, for something small.** "Monad" here means the three answers in the
section above and nothing else: no laws to check, no `do` notation, no theory required. It is still a
word that will cost you a reader, and a piece of CGP writing is usually better off describing the
behaviour than naming it.

**A `?` in a function body is simpler, and usually right.** This pays when the *steps are chosen by
wiring* — when different contexts run different chains, or the chain is assembled from parts that do not
know each other. When the steps are fixed, write a function and use `?`.

**The type errors are among CGP's worst.** A stage whose output does not match the next stage's input
fails inside the monad machinery, in terms of `Output` associated types and monad traits, and the
message rarely names the stage. Building a long pipeline one stage at a time is the practical defence.

**And nesting monads compounds that.** A stacked monad over a nested `Result` is expressive and is the
point at which a reader who has followed everything else will stop being able to predict what the
pipeline does.

## Where to go next

[Handlers](./handlers.md) is the family this composes, and the page to read first if `PipeMonadic` above
looked like it came from nowhere. [Dispatching](./dispatching.md) is the other big user of the ok monad:
matching an enum variant is a chain of attempts where the first success ends it.

For the constructs, [monad providers](/docs/reference/providers/monad_providers) carries `PipeMonadic`,
the three markers, and the per-step `BindOk` / `BindErr` forms, and
[the monad traits](/docs/reference/traits/monad) is the layer defining what a monad is here.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
