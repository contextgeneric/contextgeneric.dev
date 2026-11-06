---
title: 'Reading cargo-cgp output: the parts of a CGP error'
sidebar_label: 'Reading the output'
sidebar_position: 4
description: 'How to read the errors cargo cgp check prints: the headline and its code, the root cause, the dependency tree, several causes at once, conflicts, and fixes.'
---

# Reading the output

An error from `cargo cgp check` should tell you what failed, why it failed, and what to change.
The command is part of the [`cargo-cgp`](./index.md) toolchain for
[Context-Generic Programming (CGP)](/docs/), and keeps ordinary compiler diagnostics while
rewriting recognized CGP failures into a few recurring forms.

Each example below was captured from a complete program. The snippets beside it show only the
relevant code, so output line numbers refer to the full file. Read through the examples to learn
the diagnostic forms, or use [Error codes](./error-codes.md) to look up a particular code.

## The parts of a rewritten error

Most errors you will see have the shape of this one, from the
[checked program on the Check page](./check.md#check-the-wiring-where-you-write-it), where `Rectangle`
lacks the `height` field its area provider reads:

```text
error[E0277]: [CGP-E001] missing field `height` on `Rectangle`
  --> src/lib.rs:29:9
   |
29 |         AreaCalculatorComponent,
   |         ^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: root cause: [CGP-E110] missing field `height` on `Rectangle`
           this is required through the dependency chain:
             [CGP-E101] consumer trait impl `CanCalculateArea` for context `Rectangle`
             └─ [CGP-E102] provider trait impl `AreaCalculator` with context `Rectangle` for provider `RectangleArea`
               └─ [CGP-E110] missing field `height` on `Rectangle`
   = help: add the field `height: f64` to `Rectangle`

For more information about this error, try `rustc --explain E0277`.
```

Read the diagnostic from its headline through the notes and suggested fix:

- **The compiler's code:** `error[E0277]` is the code the compiler assigned, and the tool always keeps it.
  A rewritten error restates the compiler's error rather than replacing it, so `rustc --explain E0277`
  still applies.
- **The headline:** it starts with a CGP code and states the cause. Here `[CGP-E001]` means a context
  cannot use a consumer trait, and the rest of the line says why.
- **The `root cause:` line:** it repeats the cause under the code of its own kind, so `[CGP-E110]` is a
  missing field. When the causes are of different kinds, this line lists them and the headline counts them.
- **The dependency chain:** it shows how the requirement reached the cause, one step per line, read from the
  top. `[CGP-E101]` is the context's consumer trait, `[CGP-E102]` is the provider wired for it, and the last
  line repeats the root cause where the chain ends.
- **The `help` line:** it names a fix, when the tool can confirm one.

When the caret is far from the cause, follow the chain before applying the suggested fix.
For example, the provider named on the `[CGP-E102]` line may be different from the one you meant
to select. Adding a field would satisfy that provider, but correcting the wiring may be what the
program needs.

The caret identifies where Rust was asked to establish the requirement. With `check_components!`,
that is the check entry; otherwise, it may be the first method call that needs the trait.
The [Check page](./check.md#try-it-on-a-deliberate-mistake) shows the call-site failure whose raw
compiler message leaves out the cause.

## Several causes in one error

A provider may lack several dependencies at once. In that case, the tool lists them under
`root causes:` and branches the tree to show how each is required. Here the struct supplies
`length`, while its provider reads `width` and `height`:

```rust
#[derive(HasField)]
pub struct Rectangle {
    pub length: f64,
}
```

```text
error[E0277]: [CGP-E001] missing fields `width` and `height` on `Rectangle`
  --> src/lib.rs:28:9
   |
28 |         AreaCalculatorComponent,
   |         ^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: root causes:
             - [CGP-E110] missing field `width` on `Rectangle`
             - [CGP-E110] missing field `height` on `Rectangle`
           this is required through the dependency chain:
             [CGP-E101] consumer trait impl `CanCalculateArea` for context `Rectangle`
             └─ [CGP-E102] provider trait impl `AreaCalculator` with context `Rectangle` for provider `RectangleArea`
               ├─ [CGP-E110] missing field `width` on `Rectangle`
               └─ [CGP-E110] missing field `height` on `Rectangle`
   = help: add the fields `width: f64` and `height: f64` to `Rectangle`

For more information about this error, try `rustc --explain E0277`.
```

Both missing fields are visible here, so you can address them together before checking again.
Plain `cargo check` reports only `width` for this program; fixing that alone would expose
`height` on a later run.

Several components can also fail because of one shared dependency. If three providers read the
same missing field, the tool reports the mistake once. It marks the first check entry with the
caret, labels the others “also required here”, and includes a chain for each component.

## A component nothing is wired for

When no provider is selected, the dependency chain ends before reaching a provider implementation.
To illustrate this, the next examples switch from the shape context `Rectangle` to `App`, a type
representing an application and its wiring. Both use the same diagnostic forms. Here `App` is
checked for `FarewellComponent`, but selects only a greeter:

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
error[E0277]: [CGP-E001] missing wiring for `FarewellComponent` on `App`
  --> src/lib.rs:31:9
   |
31 |         FarewellComponent,
   |         ^^^^^^^^^^^^^^^^^
   |
   = note: root cause: [CGP-E113] missing wiring for `FarewellComponent` on `App`
           this is required through the dependency chain:
             [CGP-E101] consumer trait impl `CanSayGoodbye` for context `App`
             └─ [CGP-E113] missing wiring for `FarewellComponent` on `App`
   = help: wire `FarewellComponent` to a provider in the wiring of `App`

For more information about this error, try `rustc --explain E0277`.
```

Because no provider was selected, this chain has no `[CGP-E102]` step. Add the missing wiring
entry, or remove the check if the context should not supply this component. If a namespace or
`open` statement routes the lookup, a `[CGP-E103]` step shows that path and the chain ends at the
missing path entry instead.

## A field with the wrong type

If the field exists but its type disagrees with the provider, the headline can state that mismatch
directly and needs no separate `root cause:` note. This `height` is a `u32`, while the area
provider expects `f64`:

```rust
#[derive(HasField)]
pub struct Rectangle {
    pub width: f64,
    pub height: u32,
}
```

```text
error[E0271]: [CGP-E004] field `height` on `Rectangle` has type `u32`, but `f64` is required
  --> src/lib.rs:29:9
   |
29 |         AreaCalculatorComponent,
   |         ^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: this is required through the dependency chain:
             [CGP-E101] consumer trait impl `CanCalculateArea` for context `Rectangle`
             └─ [CGP-E102] provider trait impl `AreaCalculator` with context `Rectangle` for provider `RectangleArea`
               └─ [CGP-E112] field `height` on `Rectangle` has type `u32`, but `f64` is required

For more information about this error, try `rustc --explain E0271`.
```

The provider named in the chain tells you where the `f64` requirement came from. Use that to
decide whether the field should change or a different provider should be selected. An
[abstract type](/docs/concepts/abstract-types) that disagrees with a provider's requirement produces
the same form under `[CGP-E005]`.

## An ordinary Rust trait that does not hold

A provider can also depend on ordinary Rust traits. At a check entry, the compiler already names
a missing ordinary bound in its headline, so the tool keeps that wording and adds a dependency
chain. In this example, `CompareScalars` needs `Scalar: Eq`, while `App` selects `f64`, which
lacks that implementation:

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
               └─ [CGP-E117] the trait bound `f64: Eq` is not satisfied

For more information about this error, try `rustc --explain E0277`.
```

The `[CGP-E117]` line identifies the ordinary bound at the end of the chain, using the compiler's
wording. If the same failure is reached through a method call, the original headline describes
the method instead, so the tool replaces it with a CGP headline:

```text
error[E0599]: [CGP-E001] the trait bound `f64: Eq` is not satisfied
```

To satisfy this requirement, select a scalar type that implements `Eq`, or relax the provider's
bound if its work does not need equality. The message identifies `f64`, but leaves you to trace
that choice back to the `UseType<f64>` wiring entry.

## Two entries that claim one key

When two wiring entries conflict, the error can point directly to them. There is no dependency
chain to follow: the two carets show the competing implementations. For example, this context
selects two providers for the same component:

```rust
delegate_components! {
    App {
        GreeterComponent: GreetHello,
        GreeterComponent: GreetGoodbye,
    }
}
```

```text
error[E0119]: [CGP-E010] conflicting wiring: `App` wires component `GreeterComponent` twice
  --> src/lib.rs:27:9
   |
26 |         GreeterComponent: GreetHello,
   |         ---------------- first implementation here
27 |         GreeterComponent: GreetGoodbye,
   |         ^^^^^^^^^^^^^^^^ conflicting implementation for `App`

For more information about this error, try `rustc --explain E0119`.
```

Remove the unintended entry to give this key a single provider. Codes `[CGP-E011]` through
`[CGP-E014]` cover related overlaps, including a path covering a longer path and a direct entry
colliding with an `open` statement.

## A fix in a `help` line

For some failures, the tool can name the missing declaration directly in a `help` line.
This [`#[cgp_fn]`](/docs/reference/macros/cgp_fn) body calls another CGP function on `self`
without declaring the trait it needs:

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
error[E0599]: [CGP-E030] the trait `PersonName` is used but not declared as a dependency
  --> src/lib.rs:10:32
   |
10 |     format!("Hello, {}!", self.person_name())
   |                                ^^^^^^^^^^^
   |
   = help: declare it as a dependency with `#[uses(PersonName)]`

For more information about this error, try `rustc --explain E0599`.
```

Here adding `#[uses(PersonName)]` beneath `#[cgp_fn]` on `greeting` supplies the missing requirement.
The same remedy applies inside a [`#[cgp_impl]`](/docs/reference/macros/cgp_impl) provider body.
Without the rewrite, the compiler refers to a method on the generated `&__Context__` parameter
and never names the attribute. Many codes from `[CGP-E030]` through `[CGP-E044]` similarly
identify a mistake in a macro's input and offer a specific correction.

## Errors that pass through unchanged

`cargo cgp check` leads with the root cause for the classes it recognizes, and the tool does not yet reshape
every class. An error with no `[CGP-Exxx]` code anywhere in it, in the headline or in a dependency chain beneath
it, is one the tool passed through, so read it as the compiler wrote it. These are the ones you are most likely
to meet:

- **A check on a generic context:** `check_components! { <T> App<T> { … } }` keeps the compiler's headline, though
  its notes name the consumer and provider traits. Checking one concrete instance, such as `App<u32>`, gets the
  usual error.
- **An unconstrained lifetime or blanket key:** a per-entry lifetime the provider uses, or a blanket entry keyed
  by its own parameter, keeps the compiler's `E0207`, whose suggested fixes do not compile. See
  [A generic that reaches nothing](/docs/reference/errors#a-generic-that-reaches-nothing).
- **Errors in code a macro generated:** a shorthand that lowers to an unsized type, such as a getter returning
  `Option<&Path>`, lands on the macro's attribute. See
  [A macro lowered something the compiler rejects](/docs/reference/errors#a-macro-lowered-something-the-compiler-rejects).
- **A bare `Error` that nothing explains:** a signature that names `Error` without
  [`#[use_type]`](/docs/reference/attributes/use_type) gets the compiler's suggestion of
  `std::error::Error`, because you may have meant that type.
- **An undeclared trait called on a generic parameter other than the context:** for `value.describe()` on a
  `value: &Value` argument, the fix is a bound on `Value` rather than a `#[uses(...)]` attribute, which only
  bounds the context. So the tool leaves the compiler's message, which names `Value`, as it is.
- **A lifetime requirement on a check entry:** a bare `E0477` gives the tool no chain to follow to the bound that
  demands `'static`.

Errors in crates outside your workspace also pass through, because the tool reshapes only your workspace's own
crates. For any error that arrives raw, the [compile-errors reference](/docs/reference/errors) starts from the
compiler's own message and works back to the mistake.

---

*An AI agent wrote this page using the CGP knowledge base, and its outputs were produced by running the
commands. See [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
