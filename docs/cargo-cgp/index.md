---
title: 'cargo-cgp: readable CGP compile errors'
sidebar_label: 'Overview'
sidebar_position: 1
description: 'cargo-cgp runs in place of cargo check and reports a CGP compile error with its root cause first, including the causes plain cargo check leaves out.'
---

# cargo-cgp

[Context-Generic Programming (CGP)](/docs/) is a language extension for Rust, with pluggable trait
implementations at compile-time. `cargo-cgp` is its error toolchain: a cargo subcommand you run in
place of `cargo check` to read a CGP compile error with its root cause first, and to see the ordinary
Rust your CGP macros generate.

The tool is optional. CGP is a library on stable Rust, and `cargo check`, `cargo build`, and
`cargo test` work on a CGP project without it. Reach for `cargo-cgp` when an error is hard to read.

## The problem it solves

**Some CGP mistakes produce a compiler error that does not contain its cause.** CGP's wiring is lazy: a
[`delegate_components!`](/docs/reference/macros/delegate_components) entry selects a provider without
checking that the context can give the provider what it needs. That check happens later, where the
trait is first used, and from there the compiler often cannot say what went wrong.

In this program, `RectangleArea` reads a `width` and a `height` field through
[`#[implicit]`](/docs/reference/attributes/implicit) arguments, and `Rectangle` has no `height`:

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

The message never mentions `height`. It describes the trait machinery CGP generated, and its closing
suggestion, calling `Rectangle::area()`, fixes a different problem. The cause is not buried in the
output: it is absent, because the compiler set the candidate implementation aside before working out
why it did not apply.

`cargo cgp check` reports the same mistake like this:

```text
error[E0599]: [CGP-E001] the consumer trait `CanCalculateArea` is not implemented for context `Rectangle`
  --> src/lib.rs:28:30
   |
28 |     println!("{}", rectangle.area());
   |                              ^^^^
   |
   = note: root cause: [CGP-E106] missing field `height` on `Rectangle`
           this is required through the dependency chain:
             [CGP-E101] consumer trait impl `CanCalculateArea` for context `Rectangle`
             └─ [CGP-E102] provider trait impl `AreaCalculator` with context `Rectangle` for provider `RectangleArea`
               └─ [CGP-E106] missing field `height` on `Rectangle`

For more information about this error, try `rustc --explain E0599`.
```

The error code and the position are unchanged. The headline says which trait `Rectangle` fails to
implement, the `root cause:` line names the missing field, and the tree beneath it shows the chain of
requirements that led there. [Reading the output](./reading-output.md) explains each part.

## How it works

**It compiles your code under Rust's next-generation trait solver, which computes the cause the
default solver leaves out.** The default solver stops at "the trait bounds were not satisfied" in the
case above; the next-generation solver works the failed requirement down to the missing
`HasField` bound. That is what gives the tool a root cause to print at all.

**It rewrites the errors it recognizes.** Where the tool knows the shape of a CGP failure, it leads with
the cause, draws the chain of requirements as a tree, and tags the message with a `[CGP-Exxx]` code
naming the class of mistake. The compiler's own code stays in the header, so `rustc --explain` still
works. Errors it does not recognize pass through as the compiler wrote them.

## The two commands

[**`cargo cgp check`**](./check.md) stands in for `cargo check` when you hit or expect a wiring error.
Everything after `check` is passed to `cargo check`, so the flags you already use work unchanged.

[**`cargo cgp expand`**](./expand.md) prints the Rust your CGP macros generated, with CGP's type-level
constructs written the way you wrote them: a field name reads `Symbol!("height")` rather than a list of
characters. Use it when you want to know what a macro produced rather than guess.

## What it costs

**Installing it takes a nightly toolchain.** The tool embeds the Rust compiler, so it needs one exact
nightly with the compiler's development libraries, which is more than a gigabyte of disk. Setup
downloads that toolchain and builds a driver binary against it. Your project does not switch
toolchains: the nightly is used only for the tool's own check.

**The first check rebuilds your dependencies.** A check builds into its own `target/cgp` directory
under that nightly, so the first run compiles every dependency once more and keeps a second set of
build artifacts beside your normal ones. Later checks reuse them.

If you work with a coding assistant, [CGP's agent skill](/docs/ai/skills) teaches it to run
`cargo cgp check` and read what comes back. That makes the error output cheaper to work through; it
does not replace reading the conclusion yourself.

## What it does not do

**It is not a build tool.** There is no `cargo cgp build`, `run`, or `test`. CGP compiles on stable Rust
1.89 or newer, so build, run, and test with ordinary cargo.

**It does not decide whether your code compiles.** A check runs a different compiler and trait solver
from your project's, so treat it as a diagnostic pass, the way Clippy runs under its own settings.
Your own toolchain's `cargo check` is the authority on whether the code builds.

**It does not reshape every error.** `cargo cgp check` leads with the root cause for the classes it
recognizes, and the tool does not yet reshape every class.
[Reading the output](./reading-output.md#errors-that-pass-through-unchanged) lists the ones that pass
through, and where to read about them instead. It also reshapes errors only in your workspace's own
crates.

**It is tested on Linux.** macOS and Windows have not been tested yet.

## Where to go next

Start with installation, then the page for what you want to do:

- [Installation](./installation.md): installing with cargo or Nix, checking the install, updating,
  and uninstalling.
- [Check](./check.md): running a check, and trying it on a deliberate mistake.
- [Reading the output](./reading-output.md): every shape of error the tool produces, part by part.
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
