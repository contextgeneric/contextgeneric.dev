---
sidebar_label: 'Overview'
sidebar_position: 0
---

# Monad interface

The traits that give a monad marker its meaning for monadic handler composition.

## Overview

A [monadic pipeline](/docs/concepts/monadic-handlers) chains handlers where each step may continue or
short-circuit, the `?`-style behaviour, expressed as composable providers. A monad in CGP is a
zero-sized marker, and these four traits are what give that marker meaning. They are plain capability
traits rather than CGP components: they have no generated provider trait and are never wired. The
[monad providers](../../providers/monad/index.md) consume them as ordinary bounds while folding a
pipeline at compile time, so you name a trait here only when defining a monad of your own. Import each
from `cgp::extra::monad::traits`.

They split into the pair that folds the pipeline and the pair that runs one step.

The **folding pair** decides the shape of the composed provider. [`MonadicBind`](monadic_bind.md) turns
a continuation into one bind step, and [`MonadicTrans`](monadic_trans.md) stacks one monad over another
so a pipeline can peel a nested `Result`.

The **step-running pair** brackets one bind step. [`ContainsValue`](contains_value.md) names the value a
monad threads forward out of a step's output, and [`LiftValue`](lift_value.md) puts a value back into
the output type on either branch.

## The ideas behind them

- [Monadic handlers](/docs/concepts/monadic-handlers): why a pipeline short-circuits and how the monads
  compose.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
