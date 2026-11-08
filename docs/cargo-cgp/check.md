---
title: 'cargo cgp check: CGP errors, root cause first'
sidebar_label: 'Check'
sidebar_position: 3
description: 'Run cargo cgp check in place of cargo check to get CGP compile errors that name the root cause, then try it on a deliberate mistake.'
---

# Check

When a CGP error is hard to read, run `cargo cgp check` where you would normally run `cargo check`.
It checks your project and rewrites recognized CGP errors to lead with their root causes.
The command belongs to [`cargo-cgp`](./index.md), the error toolchain for
[Context-Generic Programming (CGP)](/docs/):

```sh
cargo cgp check
```

You can run it anywhere inside your package or workspace. Everything after `check` is passed to
`cargo check`, so the flags you already use work unchanged:

```sh
cargo cgp check --workspace
cargo cgp check -p my-crate
cargo cgp check --all-targets
```

## Try it on a deliberate mistake

A missing context field provides a small example you can try. Create a library package to hold it:

```sh
cargo new --lib area-demo
cd area-demo
```

Then add CGP to its `Cargo.toml`:

```toml
[dependencies]
cgp = "0.8.0"
```

Now replace `src/lib.rs` with this program. `RectangleArea` reads `width` and `height` through
[`#[implicit]`](/docs/reference/attributes/implicit) arguments, but `Rectangle` has only a `width`:

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

After cargo's progress lines, the check prints this error, followed by cargo's one-line summary:

```text
$ cargo cgp check

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
error: could not compile `area-demo` (lib) due to 1 previous error
```

Read the tree to connect the suggested fix to the failing call. `CanCalculateArea` is required,
so the chosen `RectangleArea` provider must work for `Rectangle`, which in turn needs `height`.
Adding the `f64` field satisfies that requirement. If the table selected the wrong provider,
however, correcting the wiring may be the intended change.

In this program, replace the comment with `pub height: f64,` and check again; it passes.
For comparison, the [overview](./index.md#the-problem-it-solves) shows the original `cargo check`
error, which never names the missing field.

## Check the wiring where you write it

The error points to `area` because the call is the first place this program requires the trait.
A wiring entry alone does not verify its provider's dependencies, so in a larger program the first
failure might occur far from that entry. Adding a
[`check_components!`](/docs/reference/macros/check_components) block beside the wiring asks for
the proof there. Replace `print_area` above with:

```rust
check_components! {
    Rectangle {
        AreaCalculatorComponent,
    }
}
```

The check now reports the same cause at the entry that asserted it:

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
error: could not compile `area-demo` (lib) due to 1 previous error
```

That explicit check helps ordinary `cargo check` expose the missing requirement too. Its field
tag still appears as a type-level string, however, with one type per character. When comparing
`height` with the existing `width` field, Rust also replaces a shared character at the same position
with `_`:

```text
help: the trait `HasField<Symbol<6, Chars<'h', Chars<'e', Chars<'i', Chars<'g', Chars<_, cgp::prelude::Chars<'t', Nil>>>>>>>>` is not implemented for `Rectangle`
```

Even with the cause exposed, the raw diagnostic runs to about thirty lines and leads with the
generated `CanUseComponent<AreaCalculatorComponent>` bound. The tool rewrites that report into
the field error and dependency chain shown above.

## What a check does differently

To recover and present these causes, the tool runs `cargo check` with its own compiler settings
and build location. The changes work together:

- **The pinned nightly:** a check compiles under the tool's own nightly, so the compiler inside the driver
  matches the one that builds your code. Your project's toolchain is untouched.
- **The next-generation trait solver:** this solver works out the missing requirement in the cases where
  the default solver only says that a method's bounds were not satisfied.
- **Its own build directory:** a check builds into `target/cgp` rather than `target`. Cargo locks a target
  directory while it builds there, so a shared one would make each check wait for your other cargo runs
  and your editor's, and the reverse.

Using a separate compiler and solver makes this a diagnostic pass rather than a reproduction
of your project's check. Like Clippy under its own settings, it can give a different result.
Mutually inheriting namespaces, for example, pass this check but are rejected by stable Rust.
Use your own toolchain's `cargo check` to decide whether the code compiles.

The build location follows the project's target directory rather than the directory you invoke
the command from. A configured `build.target-dir` or `CARGO_TARGET_DIR` therefore moves the tool's
artifacts too: with `CARGO_TARGET_DIR=/build`, they go in `/build/cgp`. Pass `--target-dir` if you
want to choose the check's directory exactly.

## Flags are cargo's, not the tool's

Since cargo receives and validates the forwarded flags, an unknown flag produces cargo's own
`error: unexpected argument '--nope' found`. Help follows the same division: `cargo cgp check --help`
displays cargo's help without running a check, while `cargo cgp --help` describes the tool's commands.

Before forwarding the arguments, the tool reads `--target-dir`, `--manifest-path`, and `--config`
to determine the build directory. Cargo still receives all three and handles their usual behavior.

## In your editor

You can also make rust-analyzer run this command whenever you save. The
[Editor integration](./editor-integration.md) page gives the VS Code setting and explains how the
override affects the editor's checks.

## When it will not help

Neither this tool nor ordinary `cargo check` reports a requirement that the program never asks
to prove. Unused, unchecked wiring can therefore remain invalid without an error. Adding
[`check_components!`](/docs/reference/macros/check_components) for each context makes those
requirements explicit.

Some errors also pass through as the compiler wrote them. `cargo cgp check` leads with the root cause for
the classes it recognizes, and the tool does not yet reshape every class.
[Reading the output](./reading-output.md#errors-that-pass-through-unchanged) lists the ones you are most
likely to meet.

Errors outside your workspace are another boundary. Dependencies compile normally, so their
errors arrive without the tool's rewrites.

If the command itself will not run, see [Troubleshooting](./troubleshooting.md).

---

*An AI agent wrote this page using the CGP knowledge base, and its outputs were produced by running the
commands. See [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
