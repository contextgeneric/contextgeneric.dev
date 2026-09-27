---
sidebar_label: 'add_mult_binary_op'
sidebar_position: 2
description: 'One generic CGP provider converts both binary operators to Lisp, reading its operands through a getter and taking the operator symbol as a type parameter.'
---

# Convert every binary operator with one provider

This **context**, the type that stands for one interpreter and holds its choices, converts
arithmetic expressions to Lisp with one provider for both `+` and `*`, where
[`add_mult`](./add-mult.md) had a provider for each. It is an example from
[`expression`](../index.md), an interpreter from the [cgp-examples](../../index.md) repository,
built with [CGP](/docs/). The change is small, and it shows how a provider can be generic over the
operator it handles.

:::tip

### New to CGP?

[`add_mult`](./add-mult.md) introduces the interpreter, its providers, and its wiring, which this
page reuses. For CGP itself, the [Hello World tutorial](/docs/tutorials/hello) introduces providers
and wiring, and [Extensible records](/docs/concepts/extensible-records) explains the field access
the new provider relies on.

:::

## Run it

This context has no test of its own in the repository, so running it takes a test of your own. Save
this as `expression/tests/binary_op.rs` in the [cgp-examples
repository](https://github.com/contextgeneric/cgp-examples):

```rust
use core::marker::PhantomData;

use cgp::extra::handler::{CanCompute, CanComputeRef};
use cgp_example_expression::contexts::add_mult_binary_op::{Interpreter, MathExpr};
use cgp_example_expression::dsl::{Eval, ToLisp};
use cgp_example_expression::types::{Literal, Plus, Times};

#[test]
fn binary_op() {
    let expr = MathExpr::Times(Times {
        left: MathExpr::Literal(Literal(2)).into(),
        right: MathExpr::Plus(Plus {
            left: MathExpr::Literal(Literal(3)).into(),
            right: MathExpr::Literal(Literal(4)).into(),
        })
        .into(),
    });

    println!("{:?}", Interpreter.compute_ref(PhantomData::<ToLisp>, &expr));
    println!("{}", Interpreter.compute(PhantomData::<Eval>, expr));
}
```

Run it from the repository root, showing what it prints:

```sh
cargo test -p cgp-example-expression --test binary_op -- --nocapture
```

It prints the Lisp tree for `(* 2 (+ 3 4))`, then the value:

```text
List(List([Ident(Ident("*")), Literal(Literal(2)), List(List([Ident(Ident("+")), Literal(Literal(3)), Literal(Literal(4))]))]))
14
```

## Two entries, one provider

The context's module,
[`add_mult_binary_op.rs`](https://github.com/contextgeneric/cgp-examples/blob/main/expression/src/contexts/add_mult_binary_op.rs),
has its own copy of `add_mult`'s enums, its context `Interpreter`, and its wrappers. Its wiring
differs in two lines, the conversion entries for the binary operators:

```rust
@ComputerRefComponent.<Code> Code.Plus<MathExpr>: BinaryOpToLisp<Symbol!("+")>,
@ComputerRefComponent.<Code> Code.Times<MathExpr>: BinaryOpToLisp<Symbol!("*")>,
```

Both entries name the same provider, `BinaryOpToLisp`, with a different type parameter.
[`Symbol!`](/docs/reference/macros/symbol) turns a string into a type, so the operator's symbol
travels in the wiring rather than in the provider's code. Evaluation is unchanged: each operator
still has its own evaluation provider, since `+` and `*` compute differently.

## The provider reads its operands through a getter

`BinaryOpToLisp` handles any input that has a `left` and a `right` operand. It says so with a getter
trait, from the crate's `providers/to_lisp/binary.rs`:

```rust
#[cgp_auto_getter]
pub trait BinarySubExpression<Expr> {
    fn left(&self) -> &Box<Expr>;
    fn right(&self) -> &Box<Expr>;
}
```

[`#[cgp_auto_getter]`](/docs/reference/macros/cgp_auto_getter) implements the trait for any type
with fields of those names and types. `Plus` and `Times` derive
[`HasField`](/docs/reference/derives/derive_has_field), which exposes their fields by name, so both
get the trait without writing an impl. Getters usually read a context's fields; this one reads the
provider's input instead.

The provider then uses the getter, the symbol, and the same local-enum trick as `add_mult`'s
conversion providers:

```rust
#[cgp_impl(new BinaryOpToLisp<Operator>)]
#[use_type(HasMathExprType.MathExpr, HasLispExprType.LispExpr)]
#[uses(CanComputeRef<Code, MathExpr, Output = LispExpr>)]
impl<Code, MathSubExpr, Operator> ComputerRef<Code, MathSubExpr>
where
    MathSubExpr: BinarySubExpression<MathExpr>,
    Operator: Default + Display,
    LispSubExpr<LispExpr>: CanUpcast<LispExpr>,
{
    type Output = LispExpr;

    fn compute_ref(&self, code: PhantomData<Code>, expr: &MathSubExpr) -> Self::Output {
        let expr_a = self.compute_ref(code, expr.left());
        let expr_b = self.compute_ref(code, expr.right());

        let ident = LispSubExpr::Ident(Ident(Operator::default().to_string())).upcast(PhantomData);

        LispSubExpr::List(List(vec![ident.into(), expr_a.into(), expr_b.into()]))
            .upcast(PhantomData)
    }
}
```

The input is any `MathSubExpr` with the getter, and `Operator` is any type that can be built and
printed, which a `Symbol!` type is: `Operator::default().to_string()` turns `Symbol!("+")` back into
`"+"`. The provider needs the context's expression type to know what the operands are, so it imports
`MathExpr` with [`#[use_type]`](/docs/reference/attributes/use_type), which the context's
`UseType<MathExpr>` entry sets.

## Try a change

Change the symbol in the `Plus` entry from `Symbol!("+")` to `Symbol!("add")`, and run the test
again. The inner operator's name changes, and nothing else does:

```text
List(List([Ident(Ident("*")), Literal(Literal(2)), List(List([Ident(Ident("add")), Literal(Literal(3)), Literal(Literal(4))]))]))
14
```

The provider was not touched; the operator's name lived only in the wiring.

## The pattern

This context shows **a provider generic over the variant it handles**, parameterized in the wiring.
Where the variants of an enum differ only in a detail, such as an operator's name, one provider can
serve them all, reading the parts it needs through a getter and taking the detail as a type
parameter. Getters over a provider's input are CGP's [extensible
records](/docs/concepts/extensible-records) put to use, and a provider taking another type as a
parameter is the same move that [higher-order providers](/docs/concepts/higher-order-providers) make
with providers.

The cost is a less obvious provider. `PlusToLisp` says what it does in its name; `BinaryOpToLisp`
needs its type parameter read in the wiring, and its bounds read in its definition, to know what it
does. For two operators the saving is small, and the generic form pays off as the operators
multiply.

## Where to go next

- [`add_mult_code`](./add-mult-code.md): the next example, both operations in one component.
- [How expression works](../architecture/index.md): where the getter and the abstract types fit in
  the design.
- [Extensible records](/docs/concepts/extensible-records): the CGP idea behind reading a struct's
  fields by name.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
project's source. See [How AI is used in this
project](/docs/ai/disclaimer#documentation-and-reference-pages).*
