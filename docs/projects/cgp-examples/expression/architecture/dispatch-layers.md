---
sidebar_label: 'Dispatch layers'
sidebar_position: 1
description: 'The two ways the expression contexts key their dispatch, by input alone or by operation and input, and why every context wraps the dispatcher for its enum.'
---

# Dispatch layers

Every context in [`expression`](../index.md) chooses a provider by the type of the expression it is
given, and some also choose by the operation. Why do they write their tables two different ways, and
why does each one write a small wrapper for its enum? `expression` is an arithmetic interpreter
built with [CGP](/docs/), and a **context** in it is a type that stands for one interpreter and
holds its choices. This page explains both arrangements, and the wrapper that every context needs.

## Two ways to pick the operation

The crate runs two operations, evaluation and conversion to Lisp, and its contexts pick between them
in one of two ways:

| Contexts | How the operation is chosen | Keys fix |
|---|---|---|
| [`add_mult`](../examples/add-mult.md), [`add_mult_binary_op`](../examples/add-mult-binary-op.md) | by which method is called: `compute` evaluates, `compute_ref` converts | the input only |
| [`add_mult_code`](../examples/add-mult-code.md), [`add_mult_neg`](../examples/add-mult-neg.md) | by a marker type passed with the input: `Eval` or `ToLisp` | the marker and the input |

In the first arrangement, each operation has a component of its own. `Interpreter` in `add_mult`
opens both, and each key's first part is a generic `<Code> Code`, which matches any marker, so only
the input decides:

```rust
open { ComputerComponent, ComputerRefComponent };

@ComputerComponent.<Code> Code.MathExpr: DispatchEval,
@ComputerComponent.<Code> Code.Plus<MathExpr>: EvalAdd,
@ComputerComponent.<Code> Code.Times<MathExpr>: EvalMultiply,
@ComputerComponent.<Code> Code.Literal<Value>: EvalLiteral,
```

In the second, one component serves every operation, so the marker has to decide. Each key fixes it,
and each operator has one entry per operation, as in `add_mult_code`:

```rust
@ComputerRefComponent.Eval.Plus<MathExpr>: EvalAdd,
@ComputerRefComponent.ToLisp.Plus<MathExpr>: BinaryOpToLisp<Symbol!("+")>,
```

`Eval` and `ToLisp` are empty structs the crate defines to name its operations. Both arrangements
are resolved by the compiler, so the choice between them changes only how the table reads and how a
caller asks for an operation. The first keeps the table short while the operations are few; the
second puts every operation behind one method and makes each operation's providers visible entry by
entry.

## Every context wraps the dispatcher

Each context maps its language enum not to CGP's dispatcher,
[`MatchWithValueHandlers`](/docs/reference/providers/dispatch/match_with_value_handlers), but to a
small provider of its own that calls it:

```rust
#[cgp_impl(new DispatchEval)]
impl<Code> Computer<Code, MathExpr> for Interpreter {
    type Output = Value;

    fn compute(context: &Interpreter, code: PhantomData<Code>, expr: MathExpr) -> Self::Output {
        <MatchWithValueHandlers>::compute(context, code, expr)
    }
}
```

The wrapper looks redundant, and it is required. The dispatcher takes a `MathExpr` apart and hands
the operator inside back to the context, and the operator's provider then asks the context to
compute its operands, which are `MathExpr` again. When the dispatcher is wired as the `MathExpr`
entry directly, the compiler has to prove "this context can evaluate a `MathExpr`" by first proving
it for each operand, which is the same question, and it never finishes. With the wiring changed to
`@ComputerComponent.<Code> Code.MathExpr: MatchWithValueHandlers`, the context's check reports that
the wiring never resolves:

```text
error[E0275]: [CGP-E010] the wiring for the consumer trait `CanCompute<Eval, MathExpr>` on context `Interpreter` never resolves — the lookup recurses without terminating
```

The same error follows for `Plus` and `Times`, and two raw overflow errors for the dispatcher
itself. Its help line suggests a component wired back to the context itself, the usual cause of the
class; here the loop runs through the operands instead.

The wrapper breaks the loop, and the likeliest reason is the shape of its impl: it is written for
one concrete context, with a concrete output type and no conditions. The compiler can then accept
that `Interpreter` evaluates a `MathExpr` from the impl's header alone, and looks into the
dispatcher only when it compiles the body, where the question no longer depends on itself.
Experiments point that way, since a wrapper that states the dispatcher's requirement as a condition
on its impl never finished compiling in them, but they are evidence rather than proof.

## What it costs

A wrapper is written per context and per operation, fixed to that context's types, so each module in
the crate defines its own `DispatchEval`, and `DispatchToLisp` where it converts. In the contexts
keyed by marker, the wrappers also fix the marker, since there the marker selects the operation.
They are a few lines each, and they are the kind of code a reader has to be told the reason for,
since nothing in them says why they exist.

Wiring through a generic dispatcher also means a missing operator entry is reported twice: once at
the context's check, which names the entry, and once at the wrapper, where the dispatcher cannot
find a provider for the variant. The check's error is the one to read, as the [`add_mult`
example](../examples/add-mult.md#try-a-change) shows.

## Where to go next

- [How expression works](./index.md): the design these tables serve.
- [`add_mult_code`](../examples/add-mult-code.md): the marker-keyed arrangement in a running
  context.
- [Dispatching](/docs/concepts/dispatching): the CGP idea behind the dispatcher, and how it checks
  every variant.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
