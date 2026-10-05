---
title: 'Reading cargo-cgp output: the parts of a CGP error'
sidebar_label: 'Reading the output'
sidebar_position: 4
description: 'How to read the errors cargo cgp check prints: the headline and its code, the root cause, the dependency tree, several causes at once, conflicts, and fixes.'
---

# Reading the output

`cargo cgp check`, part of the [`cargo-cgp`](./index.md) toolchain for
[Context-Generic Programming (CGP)](/docs/), prints ordinary compiler diagnostics, with the CGP errors
it recognizes rewritten into a few fixed shapes. This page takes each shape apart. Every example comes
from running the tool on a complete program; the snippets show the part of it that matters, and the
line numbers in the output refer to the whole file. Every `[CGP-Exxx]` code is listed on
[Error codes](./error-codes.md).

## The parts of a rewritten error

Most errors you will see have the shape of this one, from the
[checked program on the Check page](./check.md#check-the-wiring-where-you-write-it), where `Rectangle`
lacks the `height` field its area provider reads:

```text
error[E0277]: [CGP-E001] the consumer trait `CanCalculateArea` is not implemented for context `Rectangle`
  --> src/lib.rs:29:9
   |
29 |         AreaCalculatorComponent,
   |         ^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: root cause: [CGP-E106] missing field `height` on `Rectangle`
           this is required through the dependency chain:
             [CGP-E101] consumer trait impl `CanCalculateArea` for context `Rectangle`
             └─ [CGP-E102] provider trait impl `AreaCalculator` with context `Rectangle` for provider `RectangleArea`
               └─ [CGP-E106] missing field `height` on `Rectangle`

For more information about this error, try `rustc --explain E0277`.
```

Read it in four parts:

- **The compiler's code**, `error[E0277]`, is the one the compiler assigned, and it is always kept. A
  rewritten error restates the compiler's error rather than replacing it, so `rustc --explain E0277`
  still applies.
- **The headline** starts with a CGP code and says what failed, in CGP's terms. Here `[CGP-E001]` means
  a context does not implement a consumer trait: `Rectangle` cannot use `CanCalculateArea`.
- **The `root cause:` line** says why, and it is the line that names the fix. Its code describes the
  cause; `[CGP-E106]` is a missing field.
- **The dependency chain** shows how the requirement reached the cause, one step per line, read from
  the top. `[CGP-E101]` is the context's consumer trait, `[CGP-E102]` is the provider wired for it, and
  the last line repeats the root cause where the chain ends.

The chain matters when the fix is not where the caret points. If the provider in the `[CGP-E102]` line
is not the one you meant to wire, the mistake is in the wiring rather than in the struct.

The caret stays where the compiler put it: on the `check_components!` entry that asserted the
requirement, or on the call that used the trait when nothing checked it first. The
[Check page](./check.md#try-it-on-a-deliberate-mistake) shows the second case, where the compiler's
own message leaves the cause out entirely.

## Several causes in one error

When one requirement fails for more than one reason, the tool lists every cause under `root causes:`
and branches the chain to show each one. Here `Rectangle` has a `length` field and neither of the two
the provider reads:

```rust
#[derive(HasField)]
pub struct Rectangle {
    pub length: f64,
}
```

```text
error[E0277]: [CGP-E001] the consumer trait `CanCalculateArea` is not implemented for context `Rectangle`
  --> src/lib.rs:28:9
   |
28 |         AreaCalculatorComponent,
   |         ^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: root causes:
             - [CGP-E106] missing field `height` on `Rectangle`
             - [CGP-E106] missing field `width` on `Rectangle`
           this is required through the dependency chain:
             [CGP-E101] consumer trait impl `CanCalculateArea` for context `Rectangle`
             └─ [CGP-E102] provider trait impl `AreaCalculator` with context `Rectangle` for provider `RectangleArea`
               ├─ [CGP-E106] missing field `height` on `Rectangle`
               └─ [CGP-E106] missing field `width` on `Rectangle`

For more information about this error, try `rustc --explain E0277`.
```

Fix every listed cause before checking again. Plain `cargo check` reports only the `width` field for
this program, so fixing what it names would surface `height` on the next run.

The same merging works in the other direction. When several components fail because of one shared
mistake, such as a single missing field that three providers read, the tool reports one error whose
headline names all three consumer traits, rather than three errors with the same cause.

## A component nothing is wired for

When the context has no wiring entry for a component at all, the chain is short, and its last line
names the entry that is missing. The examples from here on use `App`, a type that stands for an
application and carries its wiring, where the examples above used `Rectangle`, the shape being
measured. The error shapes are the same for both. Here `App` is checked for `FarewellComponent` but
wires only the greeter:

```rust
delegate_components! {
    App {
        GreeterComponent: GreetHello,
    }
}

check_components! {
    App {
        GreeterComponent,
        FarewellComponent,
    }
}
```

```text
error[E0277]: [CGP-E001] the consumer trait `CanSayGoodbye` is not implemented for context `App`
  --> src/lib.rs:31:9
   |
31 |         FarewellComponent,
   |         ^^^^^^^^^^^^^^^^^
   |
   = note: root cause: [CGP-E107] context `App` does not contain any delegate entry for `FarewellComponent`
           this is required through the dependency chain:
             [CGP-E101] consumer trait impl `CanSayGoodbye` for context `App`
             └─ [CGP-E107] context `App` does not contain any delegate entry for `FarewellComponent`

For more information about this error, try `rustc --explain E0277`.
```

There is no `[CGP-E102]` line, because no provider was selected for the chain to pass through. Add the
entry, or remove the component from the check if the context is not meant to have it. When the
component is routed through a namespace or an `open` statement, the chain gains a `[CGP-E104]` line
naming the path the lookup followed, and the last line names that path rather than the component.

## A field with the wrong type

When the field exists but has the wrong type, the headline states the whole cause, so the error has
no separate `root cause:` line. Here `height` is a `u32` where the provider reads an `f64`:

```rust
#[derive(HasField)]
pub struct Rectangle {
    pub width: f64,
    pub height: u32,
}
```

```text
error[E0271]: [CGP-E003] expected a `height` field of type `f64` on `Rectangle`, but found `u32`
  --> src/lib.rs:29:9
   |
29 |         AreaCalculatorComponent,
   |         ^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: this is required through the dependency chain:
             [CGP-E101] consumer trait impl `CanCalculateArea` for context `Rectangle`
             └─ [CGP-E102] provider trait impl `AreaCalculator` with context `Rectangle` for provider `RectangleArea`
               └─ [CGP-E109] field `height` on `Rectangle` has type `u32`, but `f64` is required

For more information about this error, try `rustc --explain E0271`.
```

The chain still shows which provider needs the type, which tells you whether to change the field or
the provider. An [abstract type](/docs/concepts/abstract-types) wired one way and required another
produces the same shape with `[CGP-E017]`.

## An ordinary Rust trait that does not hold

Not every requirement is a CGP one. When a provider needs an ordinary trait that the wired type does
not implement, the compiler's own headline already states the cause, so the tool keeps it and adds
only the chain. Here `CompareScalars` requires `Scalar: Eq`, and `App` wires the scalar type to
`f64`, which is not `Eq`:

```rust
#[cgp_impl(new CompareScalars)]
#[use_type(HasScalarType.Scalar)]
impl ScalarEquality
where
    Scalar: Eq,
{
    fn same(&self, a: &Scalar, b: &Scalar) -> bool {
        a == b
    }
}

delegate_components! {
    App {
        ScalarTypeProviderComponent: UseType<f64>,
        ScalarEqualityComponent: CompareScalars,
    }
}
```

```text
error[E0277]: the trait bound `f64: Eq` is not satisfied
  --> src/lib.rs:36:9
   |
36 |         ScalarEqualityComponent,
   |         ^^^^^^^^^^^^^^^^^^^^^^^ the trait `Eq` is not implemented for `f64`
   |
   = note: this is required through the dependency chain:
             [CGP-E101] consumer trait impl `CanCompareScalars` for context `App`
             └─ [CGP-E102] provider trait impl `ScalarEquality` with context `App` for provider `CompareScalars`
               └─ the trait bound `f64: Eq` is not satisfied

For more information about this error, try `rustc --explain E0277`.
```

The last line of the chain repeats the compiler's words without a code, since it is not a CGP
requirement. Reached through a method call rather than a check, the same mistake gets a CGP headline
instead, and then the cause line carries `[CGP-E201]`, the code for an ordinary bound:

```text
   = note: root cause: [CGP-E201] the trait bound `f64: Eq` is not satisfied
```

The fix is in the types rather than the wiring: wire the scalar type to one that implements the trait,
or relax the provider's bound if it asks for more than it needs. The message names `f64` but not the
`UseType<f64>` entry that chose it, so that last step is yours.

## Two entries that claim one key

A wiring conflict is reported where it happens, with two carets and no chain. Here one context wires
the same component twice:

```rust
delegate_components! {
    App {
        GreeterComponent: GreetHello,
        GreeterComponent: GreetGoodbye,
    }
}
```

```text
error[E0119]: [CGP-E004] duplicate wiring for component `GreeterComponent` on `App`
  --> src/lib.rs:27:9
   |
26 |         GreeterComponent: GreetHello,
   |         ---------------- first implementation here
27 |         GreeterComponent: GreetGoodbye,
   |         ^^^^^^^^^^^^^^^^ conflicting implementation for `App`

For more information about this error, try `rustc --explain E0119`.
```

The two carets point at the two entries; remove the one you did not mean. The neighbouring codes
`[CGP-E005]` to `[CGP-E008]` cover the other ways two entries can claim one key, such as a path key
that covers a longer one, or an entry that collides with an `open` statement.

## A fix in a `help` line

Some mistakes have one right fix, and the tool states it in a `help` line. Here a
[`#[cgp_fn]`](/docs/reference/macros/cgp_fn) function calls another CGP function on `self` without
declaring it as a dependency:

```rust
#[cgp_fn]
fn person_name(&self, #[implicit] name: &str) -> String {
    name.to_owned()
}

#[cgp_fn]
fn greeting(&self) -> String {
    format!("Hello, {}!", self.person_name())
}
```

```text
error[E0599]: [CGP-E012] the trait `PersonName` is used but not declared as a dependency
  --> src/lib.rs:10:32
   |
10 |     format!("Hello, {}!", self.person_name())
   |                                ^^^^^^^^^^^
   |
   = help: declare it as a dependency with `#[uses(PersonName)]`

For more information about this error, try `rustc --explain E0599`.
```

Adding `#[uses(PersonName)]` under `#[cgp_fn]` on `greeting` fixes it. The compiler's own message for
this mistake talks about a method that exists for `&__Context__`, a type parameter the macro
generated, and never names the missing attribute. The codes from `[CGP-E012]` to `[CGP-E016]` all work
this way, each for a different attribute or trait mistake.

## Errors that pass through unchanged

`cargo cgp check` leads with the root cause for the classes it recognizes, and the tool does not yet
reshape every class. An error with no `[CGP-Exxx]` code anywhere in it, in the headline or in a
dependency chain beneath it, is one the tool passed through, so read it as the compiler wrote it.
These are the ones you are most likely to meet:

- **A per-entry generic that never reaches the key**, which the compiler reports as `E0207` twice, with
  two suggested fixes of which only the second matches the mistake. See
  [A generic that reaches nothing](/docs/reference/errors#a-generic-that-reaches-nothing).
- **Errors in code a macro generated**, such as a shorthand that lowers to an unsized type, or an
  imported type name that does not exist. These land on the macro's attribute. See
  [A macro lowered something the compiler rejects](/docs/reference/errors#a-macro-lowered-something-the-compiler-rejects).
- **Failures of the extensible-data operations**: casts between enums, builders, extractors, and
  dispatch over enum variants. Their errors name generated helper types rather than the variant or
  field at fault. [Extensible variants](/docs/concepts/extensible-variants) and
  [extensible records](/docs/concepts/extensible-records) explain what those operations require.
- **An undeclared trait called inside a [`#[cgp_impl]`](/docs/reference/macros/cgp_impl) provider's
  body.** The same mistake in a `#[cgp_fn]` body is reshaped, as shown above; inside a provider it
  arrives as the compiler's `E0599` about `&__Context__`. The fix is the same `#[uses(...)]`
  attribute.

Errors in crates outside your workspace also pass through, because the tool reshapes only your
workspace's own crates. For any error that arrives raw, the
[compile-errors reference](/docs/reference/errors) starts from the compiler's own message and works
back to the mistake.

---

*An AI agent wrote this page using the CGP knowledge base, and its outputs were produced by running the
commands. See [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
