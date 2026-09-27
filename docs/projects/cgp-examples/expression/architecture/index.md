---
sidebar_label: 'Overview'
sidebar_position: 0
description: 'How the expression interpreter works: operators as generic structs, one provider per operator, abstract output types, and contexts that wire them up.'
---

# How expression works

How can an interpreter gain a new operator without anyone editing the code that handles the old
ones? [`expression`](../index.md) is an arithmetic interpreter built with [CGP](/docs/), a language
extension for Rust with pluggable trait implementations at compile-time, and its answer is to split
the usual enum and `match` into parts that can each be reused. This page explains the design in
outline: what an operator is, what an operation is, and what puts them together.

## Each operator is its own type

A plain interpreter defines its language as one enum, with the operators as its variants. Here each
operator is a struct of its own, generic over the expression type it contains:

```rust
#[derive(Debug, Eq, PartialEq, HasField)]
pub struct Plus<Expr> {
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}
```

`Plus`, `Times`, `Minus`, and `Negate` hold boxed operands of the type `Expr`, and `Literal<T>`
holds a value of any type. None of them knows which language it belongs to. A language is then an
ordinary enum that wraps them and fills in the type parameter with itself, such as
`Plus(Plus<MathExpr>)`. So one `Plus` serves every language that has addition, and so does every
provider written for it.

## Each operation is one provider per operator

An operation over the language, such as evaluation, is written as one provider per operator:
`EvalAdd` for `Plus`, `EvalMultiply` for `Times`, and so on. Each implements one of CGP's
computation interfaces, [`Computer`](/docs/reference/components/handler/computer) or its borrowing
form [`ComputerRef`](/docs/reference/components/handler/computer_ref), for its operator, and handles
its operands by asking the context to compute them. So a provider never names the enum, or the
other operators, and it works for any language whose operands it can compute.

The crate has two operations. Evaluation turns an expression into a number, and conversion to Lisp
turns it into an S-expression tree such as `(* 2 (+ 3 4))`. The evaluation providers implement both
computation interfaces, so a context can evaluate by value or by reference with the same providers.

## A provider builds a type the context chooses

The conversion providers have to produce a Lisp expression, and they do it without naming the Lisp
enum. The context declares which concrete type its Lisp expressions are, through an [abstract
type](/docs/concepts/abstract-types), and each provider asks for it with
[`#[use_type]`](/docs/reference/attributes/use_type). The provider builds only the variants it
needs, in a small enum of its own, and converts that into the full type with
[`upcast`](/docs/reference/traits/casting/can_upcast), which works because the small enum's variants
are a subset of the full one's. So `PlusToLisp` serves any Lisp type that has a list and an
identifier.

## A context puts the pieces together

A **context** is a type that stands for one interpreter and holds its choices. The crate has four,
each an empty struct in its own module, with its own copy of the enums it uses. A context's wiring
table maps each operator type to its provider, and maps the whole language enum to a small wrapper
that calls CGP's generic dispatcher,
[`MatchWithValueHandlers`](/docs/reference/providers/dispatch/match_with_value_handlers). The
dispatcher finds which variant an expression holds and hands the operator inside back to the
context, whose table sends it on to the operator's provider. That is the `match`, written once,
generically, by CGP.

The compiler resolves every choice in the table while it type-checks the program, so none of this
is looked up at run time. [Dispatch layers](./dispatch-layers.md) explains the two ways the contexts
key their tables, and why each needs its own wrapper.

## What the design costs

The flexibility is paid for in parts. Where a plain interpreter has an enum and a `match` per
operation, this one has an operator struct per operator, a provider per operator per operation, a
wiring table per interpreter, and a wrapper per interpreter per operation. A reader tracing what
happens to a `Plus` follows it from the enum to the table to the provider, where the plain version
has one `match` arm. And a mistake in the wiring is a compile error whose chain runs through CGP's
dispatcher.

For a language that will not grow, the enum and `match` is simpler and better. The design earns its
cost when the language, or the set of operations over it, is meant to grow from code that cannot
edit the original, such as a second crate adding an operator.

## Where to go next

- [Dispatch layers](./dispatch-layers.md): how the contexts key their dispatch, and the wrapper
  every context needs.
- [`add_mult`](../examples/add-mult.md): the design in the first context, with its tests.
- [Extensible variants](/docs/concepts/extensible-variants): the CGP idea behind handling one
  variant of any enum.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
