---
sidebar_label: 'Overview'
sidebar_position: 0
description: 'The four expression contexts as short tutorials on the extensible visitor pattern: per-operator providers, a generic one, two-key dispatch, and extension.'
---

# expression examples

These pages walk through the four **contexts**, the types that each stand for one interpreter and
hold its choices, in the `expression` crate of the [cgp-examples
repository](https://github.com/contextgeneric/cgp-examples), one context per page.
[`expression`](../index.md) is an arithmetic interpreter built with [CGP](/docs/), in which each
operator is its own type and each operation is one provider per operator. Each page runs its
context, explains how the wiring produces what it computes, and names the CGP pattern it shows.

The pages are in the order they teach, and each later page changes one thing about the one before.
Start with [`add_mult`](./add-mult.md), which introduces every piece the others reuse.

:::tip

### New to CGP?

These pages explain the interpreter without deriving CGP from first principles. The [Hello World
tutorial](/docs/tutorials/hello) introduces contexts, providers, and wiring in a few minutes, and
[Extensible variants](/docs/concepts/extensible-variants) explains how CGP takes an enum apart for a
provider. Neither is required reading; each page links what it relies on.

:::

## Running an example

Clone the repository and run the tests from its root with `cargo test -p cgp-example-expression`.
Two contexts have tests of their own; the other two pages give a test to add and the output it
prints.

## The examples

- [`add_mult`](./add-mult.md) — evaluation and conversion to Lisp, with one provider per operator
  and a context that wires each operator to its provider.
- [`add_mult_binary_op`](./add-mult-binary-op.md) — one generic provider replacing the two
  per-operator conversion providers.
- [`add_mult_code`](./add-mult-code.md) — both operations in one component, chosen by an operation
  marker and the input type together.
- [`add_mult_neg`](./add-mult-neg.md) — the language extended with subtraction and negation, with
  no existing provider edited.

## Where to go next

- [How expression works](../architecture/index.md): the design behind every context, on one page.
- [Dispatching](/docs/concepts/dispatching): the CGP idea behind handling each variant with its own
  provider.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
