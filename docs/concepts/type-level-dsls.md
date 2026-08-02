---
sidebar_label: 'Type-level DSLs'
sidebar_position: 16
---

# Type-level DSLs

Encoding a small language as Rust types and interpreting it during compilation, with no interpreter
left at runtime.

This page answers *what happens when the `Code` tag carries a whole program?* It is the furthest thing
CGP's pieces are put to, and it assumes [handlers](./handlers.md). It shows a small language built from
types, why separating its syntax from its meaning is the point, and how a third party extends it. It
closes on the boundary — the programs this cannot express.

## A program that is a type

The [handler family](./handlers.md) threads a phantom `Code` tag so one context can host many
computations. Nothing says the tag has to be small.

Make it the program:

```rust
pub struct Literal<const N: u64>;
pub struct Add<Left, Right>(pub PhantomData<(Left, Right)>);
pub struct Multiply<Left, Right>(pub PhantomData<(Left, Right)>);
```

Three structs with no data and no methods. They exist to be *named in a type*, and nesting them writes a
program:

```rust
type Program = Multiply<Add<Literal<2>, Literal<3>>, Literal<4>>;
```

That is `(2 + 3) * 4` as a type. It is the language's **abstract syntax**, and — this is the whole idea —
it says nothing at all about what any of it means.

For a real language the fragments carry more: a
[`Product!`](/docs/reference/macros/product) list for a pipeline of stages, a
[`Symbol!`](/docs/reference/macros/symbol) where a string literal would go, since a `&str` value cannot
appear where only types are allowed.

## The meaning is a set of providers

Meaning arrives separately, one provider per fragment, each matching its own shape of `Code`:

```rust
#[cgp_new_provider]
impl<Context, Left, Right> Computer<Context, Add<Left, Right>, ()> for EvalAdd
where
    Context: CanCompute<Left, (), Output = u64> + CanCompute<Right, (), Output = u64>,
{
    type Output = u64;

    fn compute(context: &Context, _code: PhantomData<Add<Left, Right>>, _input: ()) -> u64 {
        context.compute(PhantomData::<Left>, ()) + context.compute(PhantomData::<Right>, ())
    }
}
```

`EvalAdd` interprets *any* addition, whatever its operands, and it evaluates them by asking the context
— which is what makes the recursion work and what keeps the provider from knowing the rest of the
language. Its requirements say only "the context can evaluate my operands".

The context then assembles the language, one row per fragment:

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

And running a program is one call:

```rust
Interpreter.compute(PhantomData::<Program>, ())   // 20
```

Because the keys are types with structure, a single row captures a whole family of programs — every
`Add<_, _>` there will ever be. A value-level lookup table cannot do that.

## What the separation buys

There is no parser, no syntax tree walked at run time, and no dispatch loop. **Type checking the call
is the interpretation**: the compiler resolves the wiring recursively through the program's structure,
and what is left in the binary is the arithmetic.

The more interesting consequence is that syntax and meaning vary independently.

**One program, several meanings.** A second context wiring the same fragments to different providers
interprets the same program differently — evaluate it, pretty-print it, cost it, run it against a test
double. Nothing in the program changes, because the program never said what it meant.

**One meaning, extended syntax.** A crate that does not own the language defines a new fragment and a
provider for it, and a context that wants both wires both. The base language is not patched, forked, or
even recompiled — the extension is a row. That is the [expression problem](./extensible-variants.md)
answered on the syntax side, and it is why a DSL built this way can have third-party dialects.

At scale the rows themselves become a [namespace](./namespaces.md), so a context joins a language rather
than listing its fragments, and an extension is a namespace inheriting the base.

## A readable surface is optional

A procedural macro can sit on top, turning an infix pipe into a nested type or a string literal into a
`Symbol!`. Such a macro does shallow token rewriting and emits the same types a programmer could write
by hand.

Keeping it that thin is deliberate: because the macro carries no meaning, the language stays fully
usable and fully extensible without it, and the grammar can evolve without touching either the macro or
the interpreters.

## What it costs

**The program must be known at compile time.** This is the boundary, and it is absolute. A script read
from a file, a pipeline configured at startup, a user-supplied expression — none of them can be a type.
If programs arrive at run time you need a runtime interpreter, and this technique is not one.

**The diagnostics are the worst CGP produces.** A malformed program is a trait-resolution failure over a
deeply nested type, reported in terms of the whole program rather than the fragment at fault. Checks
localize it and the toolchain reshapes what it recognizes; a badly nested program is still hard reading.

**Compile times feel it.** Every fragment of every program is resolution work, and a large program
instantiated several ways is where CGP's compile-time cost is most visible.

**And it is the most advanced thing here.** Everything else in this section is worth reaching for
routinely. This is worth reaching for when a language is genuinely the right shape for a problem —
build pipelines, protocol descriptions, shell-like scripting — and when the programs are fixed when the
binary is.

## Where to go next

[Handlers](./handlers.md) is the family this rests on, and the page to read first.
[Dispatching](./dispatching.md) is the same routing applied to data rather than to programs, and
[namespaces](./namespaces.md) is how a language is packaged once it outgrows a table.

For the constructs, [`Computer`](/docs/reference/components/computer) and
[`Handler`](/docs/reference/components/handler) are the interpreter interfaces,
[`delegate_components!`](/docs/reference/macros/delegate_components) carries the `open` statement and
generic path keys, and [`Product!`](/docs/reference/macros/product) and
[`Symbol!`](/docs/reference/macros/symbol) are how a real language carries lists and strings.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
