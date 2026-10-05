---
sidebar_label: 'add_mult'
sidebar_position: 1
description: 'An arithmetic interpreter that evaluates and converts expressions with one CGP provider per operator, wired by a context, with no hand-written match.'
---

# Evaluate an expression without a `match`

This context evaluates arithmetic expressions such as `2 * (3 + 4)` and converts them to Lisp, with
one provider per operator and no hand-written `match` over the language. It is the first example in
[`expression`](../index.md), an interpreter from the [cgp-examples](../../index.md) repository,
built with [CGP](/docs/). Each later example changes one thing about this one, so every piece it
introduces comes back.

:::tip

### New to CGP?

The [Hello World tutorial](/docs/tutorials/hello) introduces contexts, providers, and wiring with a
smaller program. [Extensible variants](/docs/concepts/extensible-variants) explains how CGP takes an
enum apart for a provider, which is the idea this page puts to work. This page can be read without
either.

:::

## The problem

The task is an interpreter for a small arithmetic language, with two operations over it: evaluating
an expression to a number, and converting it to Lisp. On its own, that is easy. What makes it
interesting is the requirement that the interpreter stay open: new operators and new operations
should be addable later, from code that cannot edit what is already written, such as a second crate.

### Without CGP

The crate keeps the ordinary Rust version in its `classic` module, and it is the right design for a
language that will not change:

```rust
pub enum Expr {
    Plus(Box<Expr>, Box<Expr>),
    Times(Box<Expr>, Box<Expr>),
    Literal(u64),
}

pub fn eval(expr: Expr) -> u64 {
    match expr {
        Expr::Plus(a, b) => eval(*a) + eval(*b),
        Expr::Times(a, b) => eval(*a) * eval(*b),
        Expr::Literal(value) => value,
    }
}
```

A new operation is easy here: another function with its own `match`. A new operator is not. It
means adding a variant to `Expr` and an arm to every `match` over it, and a crate that does not own
`Expr` cannot do it at all. The other common Rust design, a trait with one method per operation and
one impl per operator, has the opposite problem: new operators are easy, and every new operation is
an edit to every impl. Keeping both directions open at once, with the compiler still checking that
every case is handled, is known as the expression problem, and it is what this page's design
addresses.

## Run it

From the root of the [cgp-examples repository](https://github.com/contextgeneric/cgp-examples):

```sh
cargo test -p cgp-example-expression add_mult::
```

The context's two tests pass:

```text
test contexts::add_mult::test::test_add_mult ... ok
test contexts::add_mult::test::test_add_mult_to_lisp ... ok
```

The first evaluates `2 + 3`, `2 * 3`, and `2 * (3 + 4)` to `5`, `6`, and `14`. The second converts
the same three expressions to Lisp, so `2 * (3 + 4)` becomes the tree for `(* 2 (+ 3 4))`.

## The language is an enum of operator structs

The context's module, [`add_mult.rs`](https://github.com/contextgeneric/cgp-examples/blob/main/expression/src/contexts/add_mult.rs),
defines the language and the Lisp output as two enums:

```rust
pub type Value = u64;

#[derive(Debug, CgpData)]
pub enum MathExpr {
    Plus(Plus<MathExpr>),
    Times(Times<MathExpr>),
    Literal(Literal<Value>),
}

#[derive(Eq, PartialEq, Debug, CgpData)]
pub enum LispExpr {
    List(List<LispExpr>),
    Literal(Literal<Value>),
    Ident(Ident),
}
```

Each variant wraps an operator struct defined elsewhere in the crate, such as `Plus<Expr>` with its
`left` and `right` fields, instantiated here at `MathExpr` itself. So the operator structs, and the
providers written for them, know nothing about this particular language.
[`#[derive(CgpData)]`](/docs/reference/derives/derive_cgp_data) exposes each enum's variants to
generic code by name, which is what lets CGP take the enum apart without a `match`.

## One provider per operator

Evaluation is written as one provider per operator. This is the one for `Plus`, from the crate's
`providers/eval/add.rs`:

```rust
#[cgp_impl(new EvalAdd)]
#[uses(CanCompute<Code, MathExpr, Output = Output>)]
impl<Code, MathExpr, Output> Computer<Code, Plus<MathExpr>>
where
    Output: Add<Output = Output>,
{
    type Output = Output;

    fn compute(
        &self,
        code: PhantomData<Code>,
        Plus { left, right }: Plus<MathExpr>,
    ) -> Self::Output {
        let output_a = self.compute(code, *left);
        let output_b = self.compute(code, *right);

        output_a + output_b
    }
}
```

`EvalAdd` implements [`Computer`](/docs/reference/components/handler/computer), CGP's interface for
a computation that takes an input and returns an output, keyed by a marker type called the `Code`.
Inside the impl, `self` is the **context**, the type that stands for this interpreter and holds its
choices. [`#[uses]`](/docs/reference/attributes/uses) declares what `EvalAdd` needs from it: the
ability to evaluate each operand. So `EvalAdd` adds two evaluated operands without knowing what kind
of expression they are, or which enum `MathExpr` is. `EvalMultiply` is the same with `*`, and
`EvalLiteral` returns a literal's value.

## The context wires each operator to its provider

`Interpreter` is the context. It has no fields, because every choice it makes is in its wiring:

```rust
pub struct Interpreter;

delegate_components! {
    Interpreter {
        open { ComputerComponent, ComputerRefComponent };

        MathExprTypeProviderComponent:
            UseType<MathExpr>,
        LispExprTypeProviderComponent:
            UseType<LispExpr>,

        @ComputerComponent.<Code> Code.MathExpr: DispatchEval,
        @ComputerComponent.<Code> Code.Plus<MathExpr>: EvalAdd,
        @ComputerComponent.<Code> Code.Times<MathExpr>: EvalMultiply,
        @ComputerComponent.<Code> Code.Literal<Value>: EvalLiteral,

        @ComputerRefComponent.<Code> Code.MathExpr: DispatchToLisp,
        @ComputerRefComponent.<Code> Code.Literal<Value>: LiteralToLisp,
        @ComputerRefComponent.<Code> Code.Plus<MathExpr>: PlusToLisp,
        @ComputerRefComponent.<Code> Code.Times<MathExpr>: TimesToLisp,
    }
}
```

The `open` statement lets the table choose a provider by the computation's parameters, and each `@`
entry makes one choice. A key has two parts after the component: the `Code`, here the generic
`<Code> Code` that matches any marker, and then the input type. So
`@ComputerComponent.<Code> Code.Plus<MathExpr>: EvalAdd` reads "to compute anything on a
`Plus<MathExpr>`, use `EvalAdd`".
[`delegate_components!`](/docs/reference/macros/delegate_components) documents the syntax.

The two operations use two components. Evaluation goes through `ComputerComponent`, which takes its
input by value, and conversion goes through
[`ComputerRefComponent`](/docs/reference/components/handler/computer_ref), which borrows it. Calling
`compute` evaluates and calling `compute_ref` converts, whatever `Code` the caller passes. The two
[`UseType`](/docs/reference/providers/use_type) entries set the context's two [abstract
types](/docs/concepts/abstract-types), type names the context fills in so that providers can use
them without knowing the concrete type. The conversion providers below read the Lisp one, and the
[next example](./add-mult-binary-op.md)'s provider reads the other.

## A small wrapper takes the enum apart

The `MathExpr` entries go to a provider the module writes for itself:

```rust
#[cgp_impl(new DispatchEval)]
impl<Code> Computer<Code, MathExpr> for Interpreter {
    type Output = Value;

    fn compute(context: &Interpreter, code: PhantomData<Code>, expr: MathExpr) -> Self::Output {
        <MatchWithValueHandlers>::compute(context, code, expr)
    }
}
```

[`MatchWithValueHandlers`](/docs/reference/providers/dispatch/match_with_value_handlers) is CGP's
generic dispatcher. It finds which variant the enum holds and hands the operator struct inside back
to the context, whose table sends it to `EvalAdd`, `EvalMultiply`, or `EvalLiteral`. That replaces
the `match` a plain interpreter would write, and the compiler still checks that every variant has a
provider.

The wrapper is there because wiring `MatchWithValueHandlers` as the `MathExpr` entry directly sends
the compiler round a loop it cannot finish: evaluating a `MathExpr` needs evaluating its operands,
which are `MathExpr` again. A wrapper written for one concrete context breaks the loop. [Dispatch
layers](../architecture/dispatch-layers.md) explains why, and every context in the crate has its
own.

## Conversion builds a type it does not name

The conversion providers produce a `LispExpr`, but they are written without naming the enum. This is
the one for `Plus`, from `providers/to_lisp/add.rs`:

```rust
#[derive(CgpData)]
enum LispSubExpr<Expr> {
    List(List<Expr>),
    Ident(Ident),
}

#[cgp_impl(new PlusToLisp)]
#[use_type(HasLispExprType.LispExpr)]
#[uses(CanComputeRef<Code, MathExpr, Output = LispExpr>)]
impl<Code, MathExpr> ComputerRef<Code, Plus<MathExpr>>
where
    LispSubExpr<LispExpr>: CanUpcast<LispExpr>,
{
    type Output = LispExpr;

    fn compute_ref(
        &self,
        code: PhantomData<Code>,
        Plus { left, right }: &Plus<MathExpr>,
    ) -> Self::Output {
        let expr_a = self.compute_ref(code, left);
        let expr_b = self.compute_ref(code, right);
        let ident = LispSubExpr::Ident(Ident("+".to_owned())).upcast(PhantomData);

        LispSubExpr::List(List(vec![ident.into(), expr_a.into(), expr_b.into()]))
            .upcast(PhantomData)
    }
}
```

[`#[use_type]`](/docs/reference/attributes/use_type) brings in `LispExpr` as an abstract type the
context defines, which the `UseType<LispExpr>` entry above sets. The provider builds only the two
variants it needs, in a small local enum, and [`upcast`](/docs/reference/traits/casting/can_upcast)
turns that into the full `LispExpr`, which works because every variant of `LispSubExpr` is also a
variant of `LispExpr`. So `PlusToLisp` serves any Lisp type that has a list and an identifier.

## The checks cover every input

The module asserts its wiring with
[`check_components!`](/docs/reference/macros/check_components), listing each operation and input
pair:

```rust
check_components! {
    Interpreter {
        ComputerComponent: [
            (Eval, MathExpr),
            (Eval, Literal<Value>),
            (Eval, Plus<MathExpr>),
            (Eval, Times<MathExpr>),
        ],
        ComputerRefComponent: [
            (ToLisp, MathExpr),
            (ToLisp, Literal<Value>),
            (ToLisp, Plus<MathExpr>),
            (ToLisp, Times<MathExpr>),
        ]
    }
}
```

`Eval` and `ToLisp` are empty marker structs the crate defines as its codes. Listing each operator,
not only `MathExpr`, is what makes a missing operator entry fail at the check, as the change below
shows.

## Try a change

Remove the evaluation entry for `Times`, the line
`@ComputerComponent.<Code> Code.Times<MathExpr>: EvalMultiply`, and run
[`cargo cgp check`](/docs/cargo-cgp/check). The check names the missing entry:

```text
error[E0277]: [CGP-E001] the consumer trait `CanCompute<Eval, Times<MathExpr>>` is not implemented for context `Interpreter`
   = note: root cause: [CGP-E107] context `Interpreter` does not contain any delegate entry for `@ComputerComponent.Eval.Times<MathExpr>`
```

A second, longer error points at the wrapper's call to `MatchWithValueHandlers`, because the
dispatcher cannot find a provider for the `Times` variant either. The first error is the one to
read. `cargo cgp check` leads with the root cause for the classes it recognizes, and the tool does
not yet reshape every class. Put the line back, and the check passes.

## The pattern

This context shows the **extensible visitor pattern**: an operation over an enum written as one
provider per variant, with a generic dispatcher in place of the `match`. Each provider handles one
operator and asks the context about the rest, so it serves every language that contains that
operator, and a new operator is a new provider and a new entry rather than an edit to every
operation. [Extensible variants](/docs/concepts/extensible-variants) explains the representation
that makes this possible, and [Dispatching](/docs/concepts/dispatching) explains the dispatchers.

The cost is the machinery. Where the plain interpreter has an enum and a `match`, this one has
operator structs, a provider per operator per operation, a wiring table, and a wrapper per context,
and a mistake is a compile error with a long chain behind it. For a language that will not grow,
the `match` is simpler and better.

## Where to go next

- [`add_mult_binary_op`](./add-mult-binary-op.md): the next example, one provider for both binary
  operators.
- [Dispatch layers](../architecture/dispatch-layers.md): the two ways the contexts key their
  dispatch, and why each needs its wrapper.
- [Extensible variants](/docs/concepts/extensible-variants): the CGP idea behind handling one
  variant of any enum.
- [Extensible data types, part 2](/blog/extensible-datatypes-part-2#evaluator-computer): the post
  that developed this interpreter. Its code predates the current design.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
