---
sidebar_label: 'Overview'
sidebar_position: 0
description: 'A modular arithmetic interpreter built with CGP: each operator is its own type, each operation one provider per operator, and new operators need no edits.'
---

# expression

`expression` is a small arithmetic interpreter in which each operator is its own type, and each
operation over the language is a set of providers, one per operator. It is one of the demonstration
crates in [cgp-examples](../index.md), built with [CGP](/docs/), a language extension for Rust with
pluggable trait implementations at compile-time. The crate shows the **extensible visitor pattern**:
new operators and new operations can be added without editing the code that handles the old ones.

## The starting point: an enum and a `match`

The crate keeps the ordinary Rust version of the interpreter as the thing it improves on. It is one
enum and one `match` per operation:

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

This is the right design for a language that never changes. Adding an operation is easy, a new
function with its own `match`, but adding an operator means editing the enum and every `match` over
it, and a crate that does not own the enum cannot add one at all. Keeping both kinds of addition
open at once is the expression problem, and it is what the rest of the crate works around.

## What the crate does instead

Each operator becomes a struct of its own, generic over the expression type it contains, so one
`Plus<Expr>` serves every language that has addition:

```rust
#[derive(Debug, Eq, PartialEq, HasField)]
pub struct Plus<Expr> {
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}
```

A language is then an ordinary enum whose variants wrap those structs. Each operation is written as
one provider per operator, such as `EvalAdd` for evaluating a `Plus`, which recurses into its
operands without knowing what language they belong to. A **context**, a type that stands for one
interpreter and holds its choices, wires each operator to its provider, and CGP takes the enum apart
for it, with no hand-written `match`. The [architecture](./architecture/index.md) explains the
design in full.

The crate runs two operations over the language: **evaluation**, which turns an expression into a
number, and **conversion to Lisp**, which turns `2 * (3 + 4)` into the S-expression
`(* 2 (+ 3 4))`.

## The examples

The crate holds four contexts, each in its own module and each a different wiring of the same
providers. Each has a page, in the order they teach:

- [`add_mult`](./examples/add-mult.md) — evaluation and conversion to Lisp, one provider per
  operator, dispatched without a `match`.
- [`add_mult_binary_op`](./examples/add-mult-binary-op.md) — one generic provider converting both
  binary operators, with the operator's symbol as a type parameter.
- [`add_mult_code`](./examples/add-mult-code.md) — both operations in one component, chosen by an
  operation marker and the input type together.
- [`add_mult_neg`](./examples/add-mult-neg.md) — the language extended with subtraction and
  negation, reusing every existing provider unchanged.

## Status

`expression` is a demonstration, not a library: it is unpublished, and nobody adds it as a
dependency. It has no binary. Its three unit tests are the way to run it, and each example page
shows how to run its context, including the two that have no test of their own. The crate uses
current CGP idioms throughout, so its code is safe to copy for the patterns it shows.

## Running it

Clone the [cgp-examples repository](https://github.com/contextgeneric/cgp-examples) and run the tests
from its root:

```sh
cargo test -p cgp-example-expression
```

The workspace builds on stable Rust 1.90 or later, and needs nothing outside the build. The three
tests pass:

```text
test contexts::add_mult::test::test_add_mult_to_lisp ... ok
test contexts::add_mult_neg::test::test_add_mult_neg ... ok
test contexts::add_mult::test::test_add_mult ... ok
```

## Where to go next

- [`add_mult`](./examples/add-mult.md): the first example, and the one to read first.
- [How expression works](./architecture/index.md): the design every context shares.
- [Extensible variants](/docs/concepts/extensible-variants): the CGP idea that lets a provider
  handle one variant of any enum.
- [Extensible data types, part 2](/blog/extensible-datatypes-part-2): the post that developed this
  interpreter. Its code predates the current design.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
