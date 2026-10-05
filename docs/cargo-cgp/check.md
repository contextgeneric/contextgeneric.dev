---
title: 'cargo cgp check: CGP errors, root cause first'
sidebar_label: 'Check'
sidebar_position: 3
description: 'Run cargo cgp check in place of cargo check to get CGP compile errors that name the root cause, then try it on a deliberate mistake.'
---

# Check

`cargo cgp check` runs `cargo check` on your project and rewrites the CGP errors it recognizes so that
each one leads with its root cause. It is part of [`cargo-cgp`](./index.md), the error toolchain for
[Context-Generic Programming (CGP)](/docs/), and you use it wherever you would use `cargo check`:

```sh
cargo cgp check
```

**Run it from your package's or workspace's root directory.** Everything after `check` is passed to
`cargo check`, so the flags you already use work unchanged:

```sh
cargo cgp check --workspace
cargo cgp check -p my-crate
cargo cgp check --all-targets
```

## Try it on a deliberate mistake

A small library with one mistake shows what the tool does. Create a package:

```sh
cargo new --lib area-demo
cd area-demo
```

Add CGP to its `Cargo.toml`:

```toml
[dependencies]
cgp = "0.8.0"
```

Then replace `src/lib.rs` with this program. `RectangleArea` reads `width` and `height` through
[`#[implicit]`](/docs/reference/attributes/implicit) arguments, and `Rectangle` has only a `width`:

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

Running the check prints this, followed by cargo's own one-line summary:

```text
$ cargo cgp check

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

**Read the `root cause:` line first.** It names the fix: add a `height: f64` field to `Rectangle`, or
wire a provider that does not read one. The tree beneath it is the chain of requirements, read from
the top: `Rectangle` must implement `CanCalculateArea`, so its provider `RectangleArea` must work for
`Rectangle`, so `Rectangle` must have the field. Replace the comment with `pub height: f64,` and the
check passes. Plain `cargo check` reports this same mistake without ever mentioning `height`; the
[overview](./index.md#the-problem-it-solves) shows its output.

## Check the wiring where you write it

**Add a [`check_components!`](/docs/reference/macros/check_components) block to move the error from
the call to the wiring.** Wiring is lazy, so without one the error appears wherever the trait is first
used, which in a larger program can be far from the line you need to change. Replace `print_area` in
the program above with:

```rust
check_components! {
    Rectangle {
        AreaCalculatorComponent,
    }
}
```

The check reports the same cause, now at the entry that asserted it:

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

A check also changes what plain `cargo check` can tell you. On this version the compiler names the
missing requirement, but spells the field name one character at a time, as a type-level string:

```text
help: the trait `HasField<Symbol<6, cgp::prelude::Chars<'h', cgp::prelude::Chars<'e', cgp::prelude::Chars<'i', cgp::prelude::Chars<'g', cgp::prelude::Chars<_, cgp::prelude::Chars<'t', Nil>>>>>>>>` is not implemented for `Rectangle`
```

That line is an excerpt from an error of about thirty lines whose headline names
`CanUseComponent<AreaCalculatorComponent>`, a trait the macros generated. The tool turns it into the
sentence above.

## What a check does differently

A check is `cargo check` with three settings the tool chooses for you:

- **It compiles under the tool's pinned nightly**, so the compiler inside the driver matches the one
  that builds your code. Your project's own toolchain is untouched.
- **It uses Rust's next-generation trait solver**, which works out the missing requirement in the cases
  where the default solver only says that a method's bounds were not satisfied.
- **It builds into `target/cgp`** rather than `target`, so a check never invalidates your normal build
  cache, and your builds never invalidate its.

Because the tool picks these settings, a check is a diagnostic pass rather than a reproduction of your
own build, the way Clippy runs under settings of its own. Your toolchain's `cargo check` remains the
authority on whether the code compiles.

**`target/cgp` is relative to the directory you run the command from.** Run from a subdirectory, the
check creates a new `target/cgp` inside that subdirectory and builds every dependency again there. Pass
`--target-dir`, or set `CARGO_TARGET_DIR`, to put the artifacts somewhere else.

## Flags are cargo's, not the tool's

The tool appends your arguments to `cargo check` and lets cargo handle them, which has three
consequences:

- **Every `cargo check` flag behaves as it normally does**, because `cargo check` is what runs.
- **cargo validates them, not cargo-cgp.** An unknown flag produces cargo's error,
  `error: unexpected argument '--nope' found`, rather than a message from the tool.
- **`cargo cgp check --help` prints cargo's help** for `cargo check` and exits without running a check.
  For the tool's own commands, run `cargo cgp --help`.

The one flag the tool reads itself is `--target-dir`, and only to decide whether to supply
`target/cgp`.

## In your editor

rust-analyzer can run the check every time you save. In VS Code, add this to your `settings.json`:

```json
{
  "rust-analyzer.check.overrideCommand": [
    "cargo", "cgp", "check", "--workspace", "--all-targets", "--message-format=json"
  ]
}
```

The command must go in `check.overrideCommand` rather than `check.command`, because it is more than one
word and must print JSON for the editor to read. The tool renders its rewritten errors in the same
JSON format as the compiler, so they appear inline like any other diagnostic, and the separate
`target/cgp` directory keeps the editor's checks from contending with your builds. Other editors that
use rust-analyzer take the same setting in their own configuration format. This setup has not yet been
tried in an editor as part of writing this page.

## When it will not help

**A check reports only what your code asks the compiler to prove.** Wiring that nothing uses and
nothing checks produces no error, from this tool or any other. That is why
[`check_components!`](/docs/reference/macros/check_components) is worth adding for every context you
wire.

**Some errors pass through as the compiler wrote them.** `cargo cgp check` leads with the root cause
for the classes it recognizes, and the tool does not yet reshape every class.
[Reading the output](./reading-output.md#errors-that-pass-through-unchanged) lists the ones that do not
change.

**It reshapes errors in your workspace's own crates only.** A dependency outside the workspace is
compiled normally, so an error inside it arrives as the compiler wrote it.

If the command itself will not run, see [Troubleshooting](./troubleshooting.md).

---

*An AI agent wrote this page using the CGP knowledge base, and its outputs were produced by running the
commands. See [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
