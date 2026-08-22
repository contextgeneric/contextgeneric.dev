---
sidebar_label: 'Handlers'
sidebar_position: 11
---

# Handlers

The family of computation components, and the three axes that tell them apart: synchronous or async,
fallible or not, and taking an input or not.

This page answers *why does CGP ship a dozen computation components instead of one?* It gives the shape
they share, the three axes that separate them, the promotion that means you only ever write the
simplest one, and how a pipeline becomes a wiring entry. It closes on when a plain function is the
better answer.

## Computation as something a context decides

Most CGP components are capabilities an application has: send email, query a user, load a file. The
handler family is for a different thing: a *computation* the application performs on a value, where
the interesting question is not "can it" but "which one, and composed how".

Everything in the family shares one shape:

> given a context, a `Code` tag, and an `Input`, produce an `Output`.

The `Code` is a phantom type carrying no data. Its job is to let one context host many different
computations, each keyed by a distinct tag, so wiring can dispatch on it. The `Output` is an
*associated* type rather than a parameter, so the provider decides what a given `Code` and `Input`
produce and downstream wiring reads that decision back.

That is the whole model. The family exists because computations differ along a few axes, and forcing
every one into the most general signature would make a pure function claim capabilities it does not
have.

## Three axes, and the names that encode them

**Synchronous or asynchronous.** A synchronous member returns immediately; an asynchronous one returns
a future and carries `Async` in its name, as `AsyncComputer` beside `Computer`.

**Infallible or fallible.** A fallible member returns a `Result` against the context's
[abstract error type](./modular-error-handling.md), and carries `Try`, as `TryComputer` beside `Computer`.

**Owned or borrowed input.** A `*Ref` variant takes `&Input` where the base takes `Input`, for a
computation that only reads its argument.

Cross those and you get the members that are documented separately:
[`Computer`](/docs/reference/components/computer) is the pure synchronous corner,
[`TryComputer`](/docs/reference/components/try_computer) the fallible synchronous one, and
[`Handler`](/docs/reference/components/handler), async *and* fallible, the general workhorse, which is
why it needs no qualifier. [`Producer`](/docs/reference/components/producer) sits slightly apart: a
computation with no input at all, producing a value from the context and a tag.

The point of the spread is that a provider declares exactly the capabilities it has. A function adding
two numbers is a `Computer`, and nothing about it should mention futures or errors.

## Write the weakest one; the rest come free

Which raises the obvious objection: if wiring wants a `Handler` and you wrote a `Computer`, have you
picked wrong? No. The inclusions between the axes are real (an infallible computation is a fallible one
that never fails, a synchronous one is an async one that never awaits), and CGP encodes each as a
combinator that lifts a provider from one member into another.

The practical result is that one definition answers the whole family:

```rust
#[cgp_computer]
fn add(a: u64, b: u64) -> u64 {
    a + b
}
```

`Add` is now callable as `compute`, `try_compute`, `compute_async`, and `handle`, plus their
by-reference forms. A body that returns a `Result` promotes the same way, with its error path surfacing
through the fallible members:

```rust
#[cgp_computer]
fn checked_add(a: u64, b: u64) -> Result<u64, String> {
    a.checked_add(b).ok_or_else(|| "overflow".to_owned())
}
```

So the rule when writing is simply: **write the weakest member that fits, and let the wiring ask for
whatever it needs.** You do not have to predict which one a caller will want.

## A pipeline is a wiring entry

Because every member is an ordinary component, composing computations is composing providers, and the
combinators that do it are [higher-order providers](./higher-order-providers.md), so a whole chain has a
name and lives in a table:

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

`PipeHandlers` runs the stages left to right, threading each output into the next input. Each stage here
reads one field from the context and folds it into the running value, so `app.compute(…, 5)` evaluates
`((5 * foo) + bar) * baz`, and the shape of the computation is a line of wiring rather than a function
body. Changing the order, dropping a stage, or swapping one for another is an edit to a type.

`ComposeHandlers` nests rather than chains, `ReturnInput` passes a value through, and the `Promote*`
adapters are the lifts from the previous section made explicit. They are catalogued together in
[handler combinators](/docs/reference/providers/handler_combinators).

## Many computations on one context

The `Code` tag stops a context being limited to one computation per member of the family. Two
tags, two providers, one table:

```rust
delegate_components! {
    App {
        open ComputerComponent;

        @ComputerComponent.Doubled.i64: ComputeDoubled,
        @ComputerComponent.Negated.i64: ComputeNegated,
    }
}
```

`App.compute(PhantomData::<Doubled>, 21)` and `App.compute(PhantomData::<Negated>, 21)` reach different
providers. Dispatch can key on the `Input` type as readily as on the `Code`. That is how
[dispatching](./dispatching.md) over extensible data works, and how a program written as types becomes
interpretable, as [type-level DSLs](./type-level-dsls.md) shows.

## What it costs

**The `PhantomData` at every call site.** `app.compute(PhantomData::<()>, input)` is the price of the
tag being a type. For a context with one computation it is pure ceremony, and it does more than anything
else to make handler code look unlike ordinary Rust.

**The family is a lot of names to hold.** Twelve or so components across three axes, plus the promotion
combinators, is more surface than any other part of CGP. The mitigation is the rule above, to write the
weakest member and ignore the rest, but reading someone else's handler wiring means knowing which
member each name belongs to.

**A pipeline in a type is harder to debug than a function body.** There is no line to step through and
no place to add a print. A wrong stage shows up as a type mismatch between two providers, phrased in
terms of an `Output` associated type, and localizing it means the per-layer checks from
[checking your wiring](./check-traits.md).

**And most computations should stay functions.** This family earns its keep when the *steps* need to be
chosen by wiring: when different contexts run different pipelines, or a program is data. A computation
with one implementation is a function, and [`#[cgp_fn]`](/docs/reference/macros/cgp_fn) is how to write
one that still reads its inputs from a context.

## Where to go next

[Monadic handlers](./monadic-handlers.md) is the next thing a pipeline needs: what happens when a stage
produces a result the chain should stop on. [Dispatching](./dispatching.md) is the family applied to the
shape of a record or an enum, and [Type-level DSLs](./type-level-dsls.md) is what it becomes when the
`Code` tag carries a whole program.

[Higher-order providers](./higher-order-providers.md) is the mechanism the combinators are built from,
and worth reading first if `PipeHandlers<Product![…]>` looked like new machinery.

For the constructs, [`Computer`](/docs/reference/components/computer),
[`TryComputer`](/docs/reference/components/try_computer),
[`Handler`](/docs/reference/components/handler), and
[`Producer`](/docs/reference/components/producer) are the corners;
[`#[cgp_computer]`](/docs/reference/macros/cgp_computer) and
[`#[cgp_producer]`](/docs/reference/macros/cgp_producer) build a provider from a function; and
[handler combinators](/docs/reference/providers/handler_combinators) is the full catalogue.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
