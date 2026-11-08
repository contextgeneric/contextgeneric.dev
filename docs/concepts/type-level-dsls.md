---
title: 'Represent a program as Rust types'
sidebar_label: 'Type-level DSLs'
sidebar_position: 16
description: 'How Rust types can represent a program, how providers give each syntax form meaning, and how the compiler resolves a whole expression through wiring.'
---

# Type-level DSLs

[Context-Generic Programming](/docs/) (CGP) extends Rust with pluggable trait implementations that
can interpret a type as a program. A type-level domain-specific language represents a program as
Rust types and gives its operations meaning through CGP providers. The program's structure can stay
the same while different contexts select different interpretations, and new syntax forms can bring
their own providers. Rust checks and resolves those choices at compile time; the selected operations
execute when called.

The [handler](./handlers.md) family's `Code` parameter gives the program type a place in a
computation's interface. An interpreter can use the structure of that type to choose and combine
providers for the program's parts.

## A program represented as a type

To describe a language this way, give each syntax form a marker type. Its parameters can then
refer to other forms, allowing expressions to nest. These fragments assume `PhantomData` and CGP's
prelude are imported:

```rust
pub struct Literal<const N: u64>;
pub struct Add<Left, Right>(pub PhantomData<(Left, Right)>);
pub struct Multiply<Left, Right>(pub PhantomData<(Left, Right)>);
```

Nesting the markers represents a compound expression:

```rust
type Program = Multiply<Add<Literal<2>, Literal<3>>, Literal<4>>;
```

The nesting describes `(2 + 3) * 4`, but the markers themselves do not evaluate it. They form the
language's **abstract syntax**, leaving providers to decide what the expression means. One
interpreter can calculate its value while another formats the same expression as text.

As the language grows, its syntax can include type-level collections and strings too.
[`Product!`](/docs/reference/macros/product) can describe a sequence of pipeline stages, while
[`Symbol!`](/docs/reference/macros/symbol) places a string in a type position. These types fix the
program's structure without preventing its handlers from receiving runtime input values.

## Providers give syntax its behavior

A provider implements one syntax form and asks the context to interpret its subexpressions.
This addition provider assumes the `CanCompute` consumer trait is imported from
`cgp::extra::handler`; the `Computer` provider trait comes from the prelude:

```rust
#[cgp_impl(new EvalAdd)]
#[uses(CanCompute<Left, (), Output = u64>, CanCompute<Right, (), Output = u64>)]
impl<Left, Right> Computer<Add<Left, Right>, ()> {
    type Output = u64;

    fn compute(&self, _code: PhantomData<Add<Left, Right>>, _input: ()) -> u64 {
        self.compute(PhantomData::<Left>, ()) + self.compute(PhantomData::<Right>, ())
    }
}
```

The bounds let `EvalAdd` handle any `Add<Left, Right>` whose operands the context can compute as
`u64`. The addition provider asks the context for those results rather than selecting their
providers itself. This leaves each operand free to use syntax introduced elsewhere, including by
another crate.

The context assembles an interpreter by selecting a provider for each syntax form. This fragment
assumes an `Interpreter` context, matching `EvalLiteral` and `EvalMultiply` providers, and an import
of `ComputerComponent`:

```rust
delegate_components! {
    Interpreter {
        open ComputerComponent;

        <const N: u64> @ComputerComponent.Literal<N>.(): EvalLiteral,
        <Left, Right> @ComputerComponent.Add<Left, Right>.(): EvalAdd,
        <Left, Right> @ComputerComponent.Multiply<Left, Right>.(): EvalMultiply,
    }
}
```

The generic entries let this small table cover nested expressions. In particular, the addition
entry applies to any `Add<Left, Right>` with unit input once its provider requirements hold.
The complete `Program` can then run through the same consumer method used for other computations:

```rust
Interpreter.compute(PhantomData::<Program>, ())   // 20
```

## What the compiler resolves

Rust resolves provider selection recursively through the program type. For `Program`, it checks
that multiplication can compute its operands, that addition can compute its operands, and that
each literal has an implementation. The resulting calls use static dispatch without a runtime
syntax-tree lookup or interpreter dispatch loop.

Resolving those calls does not execute their bodies. `compute` is an ordinary function rather than
a `const fn`, so type checking does not evaluate its arithmetic. The optimizer may fold this
constant example, but the DSL makes no such guarantee. If a provider performs I/O or processes a
runtime input, that work happens when the program runs.

## Syntax and interpretation can vary independently

With the syntax represented independently, another context can interpret the same program by
pretty-printing it or estimating its cost instead of evaluating it. Each interpretation needs
providers whose input and output types fit together, but it can leave the program type unchanged.

The same separation lets an independent crate introduce another marker and a provider for it.
A context then includes the new syntax through wiring, without changing the base language's source.
Because `EvalAdd` asks only for computations with compatible outputs, the new form can become one
of its operands as soon as it meets those requirements. This is how the design addresses the
syntax-extension side of the [expression problem](./extensible-variants.md).

To reuse the interpreter's wiring, a namespace can package its shared entries and a child can
add entries for new forms. Replacing an interpretation requires more planning: under the
[namespace rule](./namespaces.md#a-bound-entry-cannot-be-overridden), a child cannot replace an
inherited binding. Any path whose provider should vary must remain open in the shared base.

## An optional surface syntax

A procedural macro can make programs easier to write by translating syntax into the same marker
types. For example, it might turn a pipeline expression into nested types or a string literal into
`Symbol!`. Interpretation remains in the providers, so handwritten types and macro-generated types
use the same implementations.

Because the macro only translates syntax, changing a provider's behavior need not change the
macro. A sufficiently generic surface syntax may also accept new fragments as they are introduced.
New grammatical forms can still require parser or macro changes, however; representing the program
as types does not by itself make every syntax extension automatic.

## What it costs

The program's type structure must be known at compile time. Runtime inputs and branches inside
providers are possible, but an arbitrary script loaded from a file cannot become a new Rust type
inside the running binary. Such a script needs a runtime interpreter or a separate compilation step.

Nesting those types also nests the requirements Rust must resolve. A missing wiring entry or an
incompatible operand output can therefore produce a long error through several provider layers.
Checking smaller expressions before combining them helps narrow down which requirement failed.

Each concrete program and interpretation also gives the compiler bounds to resolve and code to
generate. Larger programs or more interpretations can increase compilation time and code size.
Fixed build pipelines and protocol descriptions may justify that cost through reuse and extensibility.
For a few fixed computations without that need, direct Rust functions are easier to maintain.

## Where to go next

These pages explain the constructs used to build and package the language:

- [Handlers](./handlers.md): Computations parameterized by `Code` and input.
- [Dispatching](./dispatching.md): Type-directed selection applied to data.
- [Namespaces](./namespaces.md): Sharing and extending interpreter wiring.
- [`Computer`](/docs/reference/components/handler/computer) and
  [`Handler`](/docs/reference/components/handler/handler): Synchronous and async fallible
  interpreter interfaces.
- [`delegate_components!`](/docs/reference/macros/delegate_components): `open` and generic path keys.
- [`Product!`](/docs/reference/macros/product) and [`Symbol!`](/docs/reference/macros/symbol):
  Type-level lists and strings.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
