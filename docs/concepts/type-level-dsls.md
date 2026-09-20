---
sidebar_label: 'Type-level DSLs'
sidebar_position: 16
---

# Type-level DSLs

A type-level domain-specific language represents a program as Rust types and uses CGP wiring to
select implementations for its operations. The compiler checks and resolves the program's structure;
the selected operations run when called. This page develops a small arithmetic language, explains
how its syntax and interpretation can vary independently, and identifies the limits of the approach.
It assumes familiarity with [handlers](./handlers.md).

## A program represented as a type

The handler family's `Code` parameter can describe a whole program. Each syntax form becomes a
marker type, and its parameters describe the program's structure. These fragments assume
`PhantomData` and CGP's prelude are imported:

```rust
pub struct Literal<const N: u64>;
pub struct Add<Left, Right>(pub PhantomData<(Left, Right)>);
pub struct Multiply<Left, Right>(pub PhantomData<(Left, Right)>);
```

Nesting the markers represents a compound expression:

```rust
type Program = Multiply<Add<Literal<2>, Literal<3>>, Literal<4>>;
```

`Program` represents `(2 + 3) * 4`. The marker types form the language's **abstract syntax**:
they describe the expression without implementing its evaluation. An evaluator can compute its
value, while a different interpreter could format the same expression as text.

Larger languages can use type-level collections and strings within their syntax.
[`Product!`](/docs/reference/macros/product) can hold a pipeline's stages, and
[`Symbol!`](/docs/reference/macros/symbol) can represent a string in a type position.
These describe the fixed program; handlers can still receive runtime values as their inputs.

## Providers give syntax its behavior

A provider implements one syntax form and asks the context to interpret its subexpressions.
This addition provider assumes the `Computer` provider trait is imported from `cgp::extra::handler`:

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

`EvalAdd` handles any `Add<Left, Right>` whose operands the context can compute as `u64`.
Its requirements describe those computations without naming the providers that perform them.
Calling the context for each operand lets the addition provider work with syntax introduced by
other crates.

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

Each generic entry covers a family of expressions. The addition entry applies to every
`Add<Left, Right>` with unit input, subject to the selected provider's requirements. Running the
program then uses the ordinary consumer method:

```rust
Interpreter.compute(PhantomData::<Program>, ())   // 20
```

## What the compiler resolves

Rust resolves provider selection recursively through the program type. For `Program`, it checks
that multiplication can compute its operands, that addition can compute its operands, and that
each literal has an implementation. The resulting calls use static dispatch without a runtime
syntax-tree lookup or interpreter dispatch loop.

The computation itself remains ordinary Rust execution. `compute` is not a `const fn`; type
checking does not evaluate its arithmetic. An optimizer may fold this constant example, but the
DSL does not guarantee that. A provider performing I/O or processing runtime input does that work
when the program runs.

## Syntax and interpretation can vary independently

Different contexts can assign different meanings to the same program type. One context may
evaluate arithmetic while another pretty-prints it or estimates its cost. Each interpretation
needs providers with mutually compatible input and output types; the syntax stays unchanged.

An independent crate can extend the syntax by defining another marker and its provider.
A context includes that form by adding wiring, without editing the base language's source.
Existing providers such as `EvalAdd` can accept the new form as an operand when it satisfies their
computation requirements. This addresses the syntax-extension side of the
[expression problem](./extensible-variants.md).

Namespaces package a language's shared wiring for reuse. A child namespace can add entries for
new forms while inheriting the base language. Alternative interpretations must still respect the
[namespace rule](./namespaces.md#a-bound-entry-cannot-be-overridden): an inherited bound entry
cannot be replaced, so paths that need different providers must remain open in the shared base.

## An optional surface syntax

A procedural macro can make programs easier to write by translating syntax into the same marker
types. For example, it might turn a pipeline expression into nested types or a string literal into
`Symbol!`. Interpretation remains in the providers, so handwritten types and macro-generated types
use the same implementations.

Separating syntax translation from execution keeps provider changes independent of the macro.
A generic surface syntax may also accept new fragments without a macro change. New grammatical
forms can still require changes to the parser or macro; type-level representation alone does not
make every syntax extension automatic.

## What it costs

The program's type structure must be known at compile time. Runtime inputs and branches inside
providers are possible, but an arbitrary script loaded from a file cannot become a new Rust type
inside the running binary. Such a script needs a runtime interpreter or a separate compilation step.

Nested program types can produce long trait-resolution errors. Missing wiring or incompatible
operand outputs may appear through several layers of provider bounds. Checking smaller expressions
before combining them helps locate the mismatch.

Compilation must resolve the bounds and generate code for the concrete programs used. Large
programs and multiple interpretations can increase compile time and generated code size. The
tradeoff is most useful when reusable, extensible program descriptions justify those costs, such
as fixed build pipelines or protocol descriptions. Direct Rust functions are simpler for a small
set of fixed computations that does not need a language of its own.

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
