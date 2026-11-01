---
title: 'cargo cgp check: CGP errors, root cause first'
sidebar_label: 'Check'
sidebar_position: 3
description: 'Run cargo cgp check in place of cargo check to get CGP compile errors that name the root cause, then try it on a deliberate mistake.'
---

# Check

When a CGP error is hard to read, the first thing to try is `cargo cgp check`. It runs `cargo check` on
your project and rewrites the CGP errors it recognizes so that each one leads with its root cause. The
command is part of [`cargo-cgp`](./index.md), the error toolchain for
[Context-Generic Programming (CGP)](/docs/), and you use it wherever you would use `cargo check`:

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

A small library with one mistake shows what the tool does. Start by creating a package:

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
```

The headline already names the problem, and the `help` line offers one fix: add a `height: f64` field to
`Rectangle`. Another fix is to wire a provider that does not read a height, and the tree shows why that
would work too. Read from the top, it says that `Rectangle` must implement `CanCalculateArea`, so its
provider `RectangleArea` must work for `Rectangle`, so `Rectangle` must have the field. Replace the
comment with `pub height: f64,` and the check passes. By contrast, plain `cargo check` reports this same
mistake without ever mentioning `height`, as the [overview](./index.md#the-problem-it-solves) shows.

## Check the wiring where you write it

So far the error has appeared at the call to `area`, because wiring is lazy: without a check, the error
appears wherever the trait is first used. In a larger program, that place can be far from the line you
need to change. A [`check_components!`](/docs/reference/macros/check_components) block moves the error to
the wiring. Replace `print_area` in the program above with:

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
```

The `check_components!` block also changes what plain `cargo check` can tell you. With the block in place,
stable Rust names the missing requirement, but it spells the field name one character at a time, as a
type-level string. Because it compares that name with the `width` field the struct does have, it also
prints the letter the two share in that position as `_`:

```text
help: the trait `HasField<Symbol<6, Chars<'h', Chars<'e', Chars<'i', Chars<'g', Chars<_, cgp::prelude::Chars<'t', Nil>>>>>>>>` is not implemented for `Rectangle`
```

That line is an excerpt from an error of about thirty lines whose headline names
`CanUseComponent<AreaCalculatorComponent>`, a trait the macros generated. The tool turns all of it into
the message above.

## What a check does differently

The rewriting is possible because a check is `cargo check` with three settings the tool chooses for you:

- **The pinned nightly:** a check compiles under the tool's own nightly, so the compiler inside the driver
  matches the one that builds your code. Your project's toolchain is untouched.
- **The next-generation trait solver:** this solver works out the missing requirement in the cases where
  the default solver only says that a method's bounds were not satisfied.
- **Its own build directory:** a check builds into `target/cgp` rather than `target`. Cargo locks a target
  directory while it builds there, so a shared one would make each check wait for your other cargo runs
  and your editor's, and the reverse.

Because the tool picks these settings, a check is a diagnostic pass rather than a reproduction of your own
build, the way Clippy runs under settings of its own. A passing check therefore does not promise a passing
`cargo check`. For example, two namespaces that inherit from each other are rejected by stable Rust but
compile under the tool. Your toolchain's `cargo check` remains the authority on whether the code compiles.

The `target/cgp` directory sits inside your project's own target directory, wherever in the project you
run the command, so it follows a configured `build.target-dir` or a set `CARGO_TARGET_DIR`. With
`CARGO_TARGET_DIR=/build`, for example, the check builds in `/build/cgp`. To choose the check's directory
exactly, pass `--target-dir`.

## Flags are cargo's, not the tool's

The tool appends your arguments to `cargo check` and lets cargo handle them, so every `cargo check` flag
behaves as it normally does. Cargo also validates them, which means an unknown flag produces cargo's
error, `error: unexpected argument '--nope' found`, rather than a message from the tool. In the same way,
`cargo cgp check --help` prints cargo's help for `cargo check` and exits without running a check. For the
tool's own commands, run `cargo cgp --help`.

The tool reads three of the flags itself, `--target-dir`, `--manifest-path`, and `--config`, and only to
decide where `target/cgp` goes. It still passes all three on to cargo.

## In your editor

The same check can run in your editor, so the errors you see there lead with their root cause too.
rust-analyzer can run `cargo cgp check` every time you save, and it takes one setting, which
[Editor integration](./editor-integration.md) gives for VS Code, along with what the switch changes.

## When it will not help

A check reports only what your code asks the compiler to prove. Wiring that nothing uses and nothing
checks produces no error, from this tool or any other, which is why
[`check_components!`](/docs/reference/macros/check_components) is worth adding for every context you wire.

Some errors also pass through as the compiler wrote them. `cargo cgp check` leads with the root cause for
the classes it recognizes, and the tool does not yet reshape every class.
[Reading the output](./reading-output.md#errors-that-pass-through-unchanged) lists the ones you are most
likely to meet.

Finally, the tool reshapes errors in your workspace's own crates only. A dependency outside the workspace
is compiled normally, so an error inside it arrives as the compiler wrote it.

If the command itself will not run, see [Troubleshooting](./troubleshooting.md).

---

*An AI agent wrote this page using the CGP knowledge base, and its outputs were produced by running the
commands. See [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
