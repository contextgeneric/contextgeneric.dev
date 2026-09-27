---
sidebar_label: 'add_mult_code'
sidebar_position: 3
description: 'Both interpreter operations served by one CGP component, each wiring entry keyed by an operation marker and the input type together.'
---

# Choose the operation with a marker type

This **context**, the type that stands for one interpreter and holds its choices, serves both
evaluation and conversion to Lisp through one component, and chooses between them by a marker type
passed with the input. It is an example from [`expression`](../index.md), an interpreter from the
[cgp-examples](../../index.md) repository, built with [CGP](/docs/). Where
[`add_mult`](./add-mult.md) chose the operation by which method was called, this context chooses it
in its wiring, by keying each entry on two things at once.

:::tip

### New to CGP?

[`add_mult`](./add-mult.md) introduces the interpreter and its wiring, which this page reuses. For
CGP itself, the [Hello World tutorial](/docs/tutorials/hello) introduces contexts and wiring, and
[Handlers](/docs/concepts/handlers) explains the `Code` marker that CGP's computation interfaces
carry.

:::

## The problem

The task is to run two operations, evaluation and conversion to Lisp, over the same language. In
[`add_mult`](./add-mult.md), each operation has a method of its own, `compute` and `compute_ref`.
That is fine for two operations, but it ties each operation to a method, and a language with many
operations would want one entry point that takes the operation as a parameter, with each operation's
providers still listed separately, operator by operator.

### Without CGP

In plain Rust, two operations are two functions, and that is the simplest design while there are
only a few. Choosing an operation by a type instead means a trait with one impl per operation, each
carrying its own `match` over the language, so the question "what does this operation do with this
operator?" is answered inside a different function for every operation. What this page shows is a
single table that answers it for every pair of operation and operator, keyed on both at once.

## Run it

This context has no test of its own in the repository, so running it takes a test of your own. Save
this as `expression/tests/code.rs` in the [cgp-examples
repository](https://github.com/contextgeneric/cgp-examples):

```rust
use core::marker::PhantomData;

use cgp::extra::handler::CanComputeRef;
use cgp_example_expression::contexts::add_mult_code::{Interpreter, MathExpr};
use cgp_example_expression::dsl::{Eval, ToLisp};
use cgp_example_expression::types::{Literal, Plus, Times};

#[test]
fn code() {
    let expr = MathExpr::Times(Times {
        left: MathExpr::Literal(Literal(2)).into(),
        right: MathExpr::Plus(Plus {
            left: MathExpr::Literal(Literal(3)).into(),
            right: MathExpr::Literal(Literal(4)).into(),
        })
        .into(),
    });

    println!("{}", Interpreter.compute_ref(PhantomData::<Eval>, &expr));
    println!("{:?}", Interpreter.compute_ref(PhantomData::<ToLisp>, &expr));
}
```

Run it from the repository root, showing what it prints:

```sh
cargo test -p cgp-example-expression --test code -- --nocapture
```

The same method, `compute_ref`, evaluates with one marker and converts with the other:

```text
14
List(List([Ident(Ident("*")), Literal(Literal(2)), List(List([Ident(Ident("+")), Literal(Literal(3)), Literal(Literal(4))]))]))
```

## The marker is part of the key

The context's module,
[`add_mult_code.rs`](https://github.com/contextgeneric/cgp-examples/blob/main/expression/src/contexts/add_mult_code.rs),
opens one component, and every entry names both the operation and the input:

```rust
pub struct Interpreter;

delegate_components! {
    Interpreter {
        open ComputerRefComponent;

        MathExprTypeProviderComponent:
            UseType<MathExpr>,
        LispExprTypeProviderComponent:
            UseType<LispExpr>,

        @ComputerRefComponent.Eval.MathExpr: DispatchEval,
        @ComputerRefComponent.Eval.Literal<Value>: EvalLiteral,
        @ComputerRefComponent.Eval.Plus<MathExpr>: EvalAdd,
        @ComputerRefComponent.Eval.Times<MathExpr>: EvalMultiply,

        @ComputerRefComponent.ToLisp.MathExpr: DispatchToLisp,
        @ComputerRefComponent.ToLisp.Literal<Value>: LiteralToLisp,
        @ComputerRefComponent.ToLisp.Plus<MathExpr>: BinaryOpToLisp<Symbol!("+")>,
        @ComputerRefComponent.ToLisp.Times<MathExpr>: BinaryOpToLisp<Symbol!("*")>,
    }
}
```

In `add_mult`, the first part of each key was the generic `<Code> Code`, which matched any marker.
Here it is a concrete marker, `Eval` or `ToLisp`, the two empty structs the crate defines for its
operations. So `@ComputerRefComponent.Eval.Plus<MathExpr>: EvalAdd` reads "to compute `Eval` on a
`Plus<MathExpr>`, use `EvalAdd`", and each operator has one entry per operation. The table chooses
by two parameters at once, and the compiler resolves both while it type-checks the call.

## The same providers, borrowed

Evaluation runs through [`ComputerRef`](/docs/reference/components/handler/computer_ref) here, the
component that borrows its input, where `add_mult` evaluated by value. The evaluation providers
implement both components, so `EvalAdd` and the rest need no change: `EvalAdd` has a by-value impl
and a by-reference impl side by side, and this context uses the second.

The wrappers for the whole enum fix their marker to match their keys. `DispatchEval` implements the
component only for `Eval`:

```rust
#[cgp_impl(new DispatchEval)]
impl ComputerRef<Eval, MathExpr> for Interpreter {
    type Output = Value;

    fn compute_ref(
        context: &Interpreter,
        code: PhantomData<Eval>,
        expr: &MathExpr,
    ) -> Self::Output {
        <MatchWithValueHandlersRef>::compute_ref(context, code, expr)
    }
}
```

`DispatchToLisp` is the same for `ToLisp`, with `LispExpr` as its output. Since the marker passes
through the dispatcher unchanged, every operand of an `Eval` computation is evaluated, and every
operand of a `ToLisp` computation is converted.

## Try a change

Remove the conversion entry for `Plus`, the line
`@ComputerRefComponent.ToLisp.Plus<MathExpr>: BinaryOpToLisp<Symbol!("+")>`, and run
[`cargo cgp check`](/docs/cargo-cgp/check). The check names the missing entry, with both parts of
its key:

```text
error[E0277]: [CGP-E001] the consumer trait `CanComputeRef<ToLisp, Plus<MathExpr>>` is not implemented for context `Interpreter`
   = note: root cause: [CGP-E107] context `Interpreter` does not contain any delegate entry for `@ComputerRefComponent.ToLisp.Plus<MathExpr>`
```

Evaluating a `Plus` still has its entry, so only conversion is broken. Two more errors point at the
`ToLisp` wrapper's call to the dispatcher, for the same reason. `cargo cgp check` leads with the
root cause for the classes it recognizes, and the tool is a v0.1.0-alpha that does not yet reshape
every class.

## The pattern

This context shows **dispatching on two parameters at once**: a wiring key that fixes both the
operation marker and the input type, so one component serves several operations and the table says,
entry by entry, which provider each operation uses on each input. It is how a context runs many
computations through one interface, which [Handlers](/docs/concepts/handlers) describes, and the key
syntax is documented with [`delegate_components!`](/docs/reference/macros/delegate_components).

The cost is a longer table. Every operator has an entry per operation, so the table grows with
their product, and each context needs a wrapper per operation. Choosing the operation by which
method is called, as `add_mult` does, keeps the table shorter when there are only two.

## Where to go next

- [`add_mult_neg`](./add-mult-neg.md): the next example, the language extended with new operators.
- [Dispatch layers](../architecture/dispatch-layers.md): the two ways the contexts key their
  dispatch, side by side.
- [Handlers](/docs/concepts/handlers): CGP's computation interfaces, and the `Code` marker they
  share.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
