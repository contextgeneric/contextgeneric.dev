---
sidebar_label: 'add_mult_neg'
sidebar_position: 4
description: 'The interpreter''s language extended with subtraction and negation in a new enum and context, reusing every existing CGP provider unchanged.'
---

# Extend the language without editing it

This **context**, the type that stands for one interpreter and holds its choices, evaluates a larger
language, with subtraction and negation added to addition and multiplication, and no existing
provider is edited to do it. It is the last example in [`expression`](../index.md), an interpreter
from the [cgp-examples](../../index.md) repository, built with [CGP](/docs/). It is the pay-off of
the design the earlier examples set up: the language grows by adding types and providers next to the
old ones.

:::tip

### New to CGP?

[`add_mult`](./add-mult.md) introduces the interpreter and its wiring, which this page extends. For
CGP itself, the [Hello World tutorial](/docs/tutorials/hello) introduces contexts and wiring, and
[Extensible variants](/docs/concepts/extensible-variants) explains how one provider can serve
several enums.

:::

## Run it

From the root of the [cgp-examples repository](https://github.com/contextgeneric/cgp-examples):

```sh
cargo test -p cgp-example-expression add_mult_neg::
```

The context's test passes:

```text
test contexts::add_mult_neg::test::test_add_mult_neg ... ok
```

It evaluates `2 + 3` to `5`, `2 * 3` to `6`, `2 - 3` to `-1`, and `-2 * (3 + 4)` to `-14`.

## A new enum, from old parts and new ones

The context's module,
[`add_mult_neg.rs`](https://github.com/contextgeneric/cgp-examples/blob/main/expression/src/contexts/add_mult_neg.rs),
defines the larger language as a new enum:

```rust
pub type Value = i64;

#[derive(Debug, CgpData)]
pub enum MathPlusExpr {
    Plus(Plus<MathPlusExpr>),
    Times(Times<MathPlusExpr>),
    Literal(Literal<Value>),
    Negate(Negate<MathPlusExpr>),
    Minus(Minus<MathPlusExpr>),
}
```

`Plus`, `Times`, and `Literal` are the same operator structs the base language uses, here
instantiated at `MathPlusExpr` and with `i64` literals, since the language now has negative
numbers. `Negate` and `Minus` are two more operator structs. The base language's `MathExpr` is left
as it was; the new enum sits beside it.

## The new operators get providers of their own

Each new operator needs an evaluation provider. `EvalNegate`, from the crate's
`providers/eval/negate.rs`, negates its one operand:

```rust
#[cgp_impl(new EvalNegate)]
#[uses(CanComputeRef<Code, MathExpr, Output = Output>)]
impl<Code, MathExpr, Output> ComputerRef<Code, Negate<MathExpr>>
where
    Output: Neg<Output = Output>,
{
    type Output = Output;

    fn compute_ref(
        &self,
        code: PhantomData<Code>,
        Negate(expr): &Negate<MathExpr>,
    ) -> Self::Output {
        -self.compute_ref(code, expr)
    }
}
```

`EvalSubtract` does the same for `Minus` with `-` between its two operands. Like every provider in
the crate, each asks the context to evaluate its operands and knows nothing about the enum they sit
in. They could be written in another crate from the one that defines the base language.

## The context mixes old providers and new

`InterpreterPlus` is a new context, with a table that wires the old providers for the old operators
and the new providers for the new ones:

```rust
pub struct InterpreterPlus;

delegate_components! {
    InterpreterPlus {
        open ComputerRefComponent;

        @ComputerRefComponent.Eval.MathPlusExpr: DispatchEval,
        @ComputerRefComponent.Eval.Plus<MathPlusExpr>: EvalAdd,
        @ComputerRefComponent.Eval.Times<MathPlusExpr>: EvalMultiply,
        @ComputerRefComponent.Eval.Literal<Value>: EvalLiteral,
        @ComputerRefComponent.Eval.Minus<MathPlusExpr>: EvalSubtract,
        @ComputerRefComponent.Eval.Negate<MathPlusExpr>: EvalNegate,
    }
}
```

`EvalAdd`, `EvalMultiply`, and `EvalLiteral` are the providers `add_mult` wires, unchanged, now at a
different enum and a different number type. The table is keyed on the `Eval` marker and the input,
as in [`add_mult_code`](./add-mult-code.md), and the context has its own `DispatchEval` wrapper for
`MathPlusExpr`, for the reason [`add_mult`](./add-mult.md#a-small-wrapper-takes-the-enum-apart)
gives.

## Conversion is left unwired

`InterpreterPlus` wires evaluation only. There is no `ToLisp` entry, and no abstract Lisp type, and
the context compiles and runs regardless: CGP checks wiring where it is used, so an operation that
nothing calls needs no providers. The context's `check_components!` block lists evaluation for all
six input types, and nothing for conversion.

## Try a change

Ask for conversion anyway. Add `ToLisp` to the module's `dsl` import, and add an entry to its check
block:

```rust
(ToLisp, MathPlusExpr),
```

Run [`cargo cgp check`](/docs/cargo-cgp/check), and the check reports that conversion was never
wired:

```text
error[E0277]: [CGP-E001] the consumer trait `CanComputeRef<ToLisp, MathPlusExpr>` is not implemented for context `InterpreterPlus`
   = note: root cause: [CGP-E107] context `InterpreterPlus` does not contain any delegate entry for `@ComputerRefComponent.ToLisp.MathPlusExpr`
```

Supporting it would be a second group of `ToLisp` entries beside the `Eval` ones, with a provider
for each new operator, an entry binding the context's Lisp type, and a `DispatchToLisp` wrapper, as
[`add_mult`](./add-mult.md) has. `cargo cgp check` leads with the root cause
for the classes it recognizes, and the tool is a v0.1.0-alpha that does not yet reshape every class.

## The pattern

This context shows **the expression problem solved**: a language extended with new variants while
every existing operation's providers stay as they were. The old operators' providers serve the new
enum because they never named an enum; the new operators get providers of their own; and a new
context wires the two together. Adding a new operation instead would be the same move in the other
direction, a new set of providers and a new group of entries. [Extensible
variants](/docs/concepts/extensible-variants) explains the representation that allows it.

The cost is that the extension is a new language, not a change to the old one. `MathPlusExpr` is a
separate enum and `InterpreterPlus` a separate context, and code that handles `MathExpr` does not
accept `MathPlusExpr` until it is wired for it. The flexibility is in the providers, which are
reused, rather than in the enums, which stay ordinary closed Rust enums.

## Where to go next

- [How expression works](../architecture/index.md): the design the four examples share, on one
  page.
- [Dispatch layers](../architecture/dispatch-layers.md): why each context, this one included, has
  its own wrapper.
- [Extensible variants](/docs/concepts/extensible-variants): the CGP idea that lets one provider
  serve several enums.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
