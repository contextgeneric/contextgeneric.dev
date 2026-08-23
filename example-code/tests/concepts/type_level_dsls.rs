//! Code from `docs/concepts/type-level-dsls.md` — *Type-level DSLs*.

use core::marker::PhantomData;

use cgp::extra::handler::{Computer, ComputerComponent};
use cgp::prelude::*;

/// ## The syntax is types
///
/// Each fragment of the language is a struct with no data. Its parameters carry the program's
/// structure, and nesting them is how a program is written.
pub struct Literal<const N: u64>;
pub struct Add<Left, Right>(pub PhantomData<(Left, Right)>);
pub struct Multiply<Left, Right>(pub PhantomData<(Left, Right)>);

/// ## One provider per fragment is the interpreter
///
/// Each provider matches one shape of `Code` and says what it means. Nothing about the fragment
/// itself decided that.
#[cgp_new_provider]
impl<Context, const N: u64> Computer<Context, Literal<N>, ()> for EvalLiteral {
    type Output = u64;

    fn compute(_context: &Context, _code: PhantomData<Literal<N>>, _input: ()) -> u64 {
        N
    }
}

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

#[cgp_new_provider]
impl<Context, Left, Right> Computer<Context, Multiply<Left, Right>, ()> for EvalMultiply
where
    Context: CanCompute<Left, (), Output = u64> + CanCompute<Right, (), Output = u64>,
{
    type Output = u64;

    fn compute(context: &Context, _code: PhantomData<Multiply<Left, Right>>, _input: ()) -> u64 {
        context.compute(PhantomData::<Left>, ()) * context.compute(PhantomData::<Right>, ())
    }
}

use cgp::extra::handler::CanCompute;

/// ## A context assembles the language
///
/// Each row maps one fragment shape to its interpreter. Adding a fragment is adding a row.
pub struct Interpreter;

delegate_components! {
    Interpreter {
        open ComputerComponent;

        <const N: u64> @ComputerComponent.Literal<N>.(): EvalLiteral,
        <Left, Right> @ComputerComponent.Add<Left, Right>.(): EvalAdd,
        <Left, Right> @ComputerComponent.Multiply<Left, Right>.(): EvalMultiply,
    }
}

mod check_interpreter {
    use super::*;
    check_components! {
        Interpreter {
            ComputerComponent: [
                (Literal<2>, ()),
                (Add<Literal<2>, Literal<3>>, ()),
                (Multiply<Add<Literal<2>, Literal<3>>, Literal<4>>, ()),
            ],
        }
    }
}

/// ## Running a program is type checking it
#[test]
fn the_program_is_a_type_and_the_answer_is_computed_at_compile_time() {
    // (2 + 3) * 4
    type Program = Multiply<Add<Literal<2>, Literal<3>>, Literal<4>>;

    assert_eq!(Interpreter.compute(PhantomData::<Program>, ()), 20);
}
