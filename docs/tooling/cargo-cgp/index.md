---
title: 'cargo-cgp: readable CGP compile errors'
sidebar_label: 'Overview'
sidebar_position: 1
description: 'cargo-cgp runs in place of cargo check and reports a CGP compile error with its root cause first, including the causes plain cargo check leaves out.'
---

# cargo-cgp

`cargo-cgp` helps you understand compile errors and macro expansions in
[Context-Generic Programming (CGP)](/docs/) code. CGP extends Rust with pluggable trait
implementations through macros that generate ordinary Rust traits, so a compiler error may name
that generated machinery while leaving the failed requirement unclear. Running `cargo cgp check`
brings the root cause forward for failures the tool recognizes.

You can keep using ordinary cargo commands alongside it. CGP is a library on stable Rust, so
`cargo check`, `cargo build`, and `cargo test` work without the tool. Reach for `cargo-cgp` when
you need help understanding an error or a macro expansion.

## The problem it solves

A wiring entry can select a provider even when its context lacks a dependency. The
[`delegate_components!`](/docs/reference/macros/delegate_components) table records the choice,
and Rust checks whether it works when the trait is required later. At that later call site, the
compiler can report unsatisfied bounds without exposing the dependency that failed.

For example, this table selects `RectangleArea` for `Rectangle`. The provider obtains `width`
and `height` through [`#[implicit]`](/docs/reference/attributes/implicit) arguments, but the
context supplies only `width`:

```rust
use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_impl(new RectangleArea)]
impl AreaCalculator {
    fn area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
        width * height
    }
}

#[derive(HasField)]
pub struct Rectangle {
    pub width: f64,
    // missing `height` field
}

delegate_components! {
    Rectangle {
        AreaCalculatorComponent: RectangleArea,
    }
}

pub fn print_area(rectangle: &Rectangle) {
    println!("{}", rectangle.area());
}
```

Plain `cargo check` on stable Rust reports the call to `area`:

```text
error[E0599]: the method `area` exists for reference `&Rectangle`, but its trait bounds were not satisfied
  --> src/lib.rs:28:30
   |
16 | pub struct Rectangle {
   | -------------------- doesn't satisfy `Rectangle: AreaCalculator<Rectangle>` or `Rectangle: CanCalculateArea`
...
28 |     println!("{}", rectangle.area());
   |                              ^^^^ this is an associated function, not a method
   |
   = note: found the following associated functions; to be used as methods, functions must have a `self` parameter
note: the candidate is defined in the trait `AreaCalculator`
  --> src/lib.rs:5:5
   |
 5 |     fn area(&self) -> f64;
   |     ^^^^^^^^^^^^^^^^^^^^^^
note: the following trait bounds were not satisfied:
      `&Rectangle: AreaCalculator<&Rectangle>`
      `Rectangle: AreaCalculator<Rectangle>`
  --> src/lib.rs:3:1
   |
 3 | #[cgp_component(AreaCalculator)]
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 4 | pub trait CanCalculateArea {
   |           ^^^^^^^^^^^^^^^^
note: the trait `AreaCalculator` must be implemented
  --> src/lib.rs:4:1
   |
 4 | / pub trait CanCalculateArea {
 5 | |     fn area(&self) -> f64;
 6 | | }
   | |_^
   = help: items from traits can only be used if the trait is implemented and in scope
note: `CanCalculateArea` defines an item `area`, perhaps you need to implement it
  --> src/lib.rs:4:1
   |
 4 | pub trait CanCalculateArea {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^
   = note: this error originates in the attribute macro `cgp_component` (in Nightly builds, run with -Z macro-backtrace for more info)
help: use associated function syntax instead
   |
28 -     println!("{}", rectangle.area());
28 +     println!("{}", Rectangle::area());
   |

For more information about this error, try `rustc --explain E0599`.
```

Nothing in this message names `height`. Instead, it reports the generated trait bounds and
suggests calling `Rectangle::area()`, which would address a different problem. The missing field
never reached the diagnostic because the compiler set the candidate implementation aside before
working out why it did not apply.

`cargo cgp check` reports the same mistake like this:

```text
error[E0599]: [CGP-E001] missing field `height` on `Rectangle`
  --> src/lib.rs:28:30
   |
28 |     println!("{}", rectangle.area());
   |                              ^^^^
   |
   = note: root cause: [CGP-E110] missing field `height` on `Rectangle`
           this is required through the dependency chain:
             [CGP-E101] consumer trait impl `CanCalculateArea` for context `Rectangle`
             └─ [CGP-E102] provider trait impl `AreaCalculator` with context `Rectangle` for provider `RectangleArea`
               └─ [CGP-E110] missing field `height` on `Rectangle`
   = help: add the field `height: f64` to `Rectangle`

For more information about this error, try `rustc --explain E0599`.
```

The rewritten headline identifies `height` immediately. Below it, the tree connects the call
to the provider and its field requirement, followed by a suggested fix. The compiler's code and
source position stay in place, as [Reading the output](./reading-output.md) explains in more detail.

## How it works

To produce this message, the tool first checks the program with Rust's next-generation trait
solver. Where the default solver stops at unsatisfied method bounds, this solver follows the
requirement down to the missing `HasField` bound. That gives the tool a cause it can report.

The tool then rewrites the failures it recognizes, naming the cause, drawing the dependency chain,
and adding a `[CGP-Exxx]` code for the class of mistake. It retains the compiler's code so
`rustc --explain` still applies. Errors it does not recognize pass through as the compiler wrote them.

## The two commands

For a wiring error, [`cargo cgp check`](./check.md) is usually the command to reach for.
It accepts the arguments you would pass to `cargo check`, so you can keep the same package,
workspace, and feature selections.

If you need to inspect the generated implementation, use [`cargo cgp expand`](./expand.md).
It prints the macro expansion with CGP's type-level constructs in their familiar forms.
For example, a field tag appears as `Symbol!("height")`, making the name readable without
following a list of character types.

## What it costs

To make those diagnostics available, setup downloads one exact nightly and builds the driver
against its compiler libraries. The toolchain occupies more than a gigabyte, so installation
has a cost in both time and disk space. That nightly belongs to the tool's diagnostic pass;
your project keeps its own toolchain.

Checking under that nightly also needs a separate build. The tool puts its artifacts in
`target/cgp`, so the first run compiles every dependency again and uses additional disk space.
Later checks reuse those artifacts alongside your normal build output.

[CGP's agent skill](/docs/ai/skills) can help a coding assistant run the check and interpret its
output. You still need to review the diagnosis and proposed change yourself.

## What it does not do

For building, running, and testing, keep using ordinary cargo. The tool provides no
`cargo cgp build`, `run`, or `test` commands, and CGP itself compiles on stable Rust 1.89 or newer.

Because this diagnostic pass uses its own compiler and trait solver, its result can differ from
your project's check. Treat it as you would a Clippy run under separate settings, and keep your
own toolchain's `cargo check` as the authority on whether the code builds.

Its diagnostic coverage has limits too. `cargo cgp check` leads with the root cause for the classes it
recognizes, and the tool does not yet reshape every class.
[Reading the output](./reading-output.md#errors-that-pass-through-unchanged) identifies common
cases that pass through and points to their explanations. Rewriting also applies only to crates
inside your workspace.

Finally, the tool is tested on Linux. macOS and Windows have not been tested yet.

## Where to go next

Start with installation, then go to the page for what you want to do:

- [Installation](./installation.md): installing with cargo or Nix, checking the install, updating,
  and uninstalling.
- [Check](./check.md): running a check, and trying it on a deliberate mistake.
- [Reading the output](./reading-output.md): the shapes of error the tool produces, part by part.
- [Expand](./expand.md): reading what your CGP macros generated.
- [Editor integration](./editor-integration.md): rewritten errors in VS Code, every time you save.
- [Troubleshooting](./troubleshooting.md): when the tool itself will not run.
- [Error codes](./error-codes.md) and [Command reference](./command-reference.md): to look something
  up.

For a guided walk through one mistake, from the raw error to the fix, the area-calculation tutorial's
[checking part](/docs/tutorials/area-calculation/checking) uses the tool on a program it builds up
step by step.

---

*An AI agent wrote this page using the CGP knowledge base, and its outputs were produced by running the
commands. See [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
