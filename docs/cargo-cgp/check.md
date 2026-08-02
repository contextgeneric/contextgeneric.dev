---
sidebar_position: 3
---

# Check

`cargo cgp check` stands in for `cargo check` and re-presents CGP's wiring errors with their root cause
first.

```sh
cargo cgp check
```

Run it from anywhere inside a cargo package or workspace that uses `cgp`. Everything after `check` is
forwarded verbatim to `cargo check`, so the flags you already use work unchanged:

```sh
cargo cgp check --workspace
cargo cgp check -p my-crate
cargo cgp check --all-targets
```

## A worked example

Here is a complete program with one mistake in it. `RectangleArea` reads a `height` field through an
[`#[implicit]`](/docs/reference/attributes/implicit) argument, and `Rectangle` does not have one —
someone removed it, or never added it.

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
}

delegate_components! {
    Rectangle {
        AreaCalculatorComponent: RectangleArea,
    }
}

check_components! {
    Rectangle {
        AreaCalculatorComponent,
    }
}
```

Running the tool on it:

```text
$ cargo cgp check

error[E0277]: [CGP-E001] the consumer trait `CanCalculateArea` is not implemented for context `Rectangle`
  --> src/lib.rs:28:9
   |
28 |         AreaCalculatorComponent,
   |         ^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: root cause: [CGP-E106] missing field `height` on `Rectangle`
           this is required through the dependency chain:
             [CGP-E101] consumer trait impl `CanCalculateArea` for context `Rectangle`
             └─ [CGP-E102] provider trait impl `AreaCalculator` with context `Rectangle` for provider `RectangleArea`
               └─ [CGP-E106] missing field `height` on `Rectangle`

For more information about this error, try `rustc --explain E0277`.
```

**Read the `root cause:` line first.** It names the fix — add a `height: f64` field to `Rectangle`, or
stop reading one. The tree beneath it is the chain of obligations that demanded it, read top to bottom:
the context has to implement the consumer trait, which means its wired provider has to be a provider
for that context, which means the context has to have the field.

For comparison, the same mistake under plain `cargo check`:

```text
help: the trait `HasField<Symbol<6, cgp::prelude::Chars<'h', cgp::prelude::Chars<'e',
      cgp::prelude::Chars<'i', cgp::prelude::Chars<'g', cgp::prelude::Chars<_,
      cgp::prelude::Chars<'t', Nil>>>>>>>>` is not implemented for `Rectangle`
```

The field name is in there, one character per type parameter. That is the class of message the tool
exists to turn into a sentence.

## Reading the output

The output is ordinary rustc and cargo diagnostics, with the errors the tool recognizes rewritten.
A rewritten message carries a short code in square brackets:

```text
error[E0277]: [CGP-E001] the consumer trait `CanCalculateArea` is not implemented for context `Rectangle`
```

The `[CGP-Exxx]` code names one class of CGP mistake. Codes appear in two places and mean slightly
different things: the one in the **headline** classifies the error, and the ones in the **root-cause
tree** label each link in the chain. In the example above, `CGP-E001` is "a context does not implement
a consumer trait", `CGP-E106` is "a required field is missing", and `CGP-E101`/`CGP-E102` are the
consumer and provider impls in between.

The compiler's own code is always kept — `E0277` here — so `rustc --explain` still works and nothing is
reclassified away from rustc. The CGP code rides inside the message as a tag on the sentence it
classifies. **Errors the tool does not recognize pass through exactly as the compiler wrote them.**

## What a check does differently

Three deliberate differences from `cargo check`, all handled for you:

- **It compiles under the tool's pinned nightly**, not your project's toolchain, so the diagnostics are
  reproducible and the embedded compiler matches the driver. Your project's toolchain is untouched.
- **It turns on the next-generation trait solver.** This is the substantive one: the default solver
  *hides* the failing bound in CGP's worst error class, reporting only that a method's bounds were not
  satisfied. The new solver names the bound, which is what makes a root cause available to print.
- **It builds into `target/cgp`** rather than your project's `target/`, so a check never invalidates
  your normal build cache and vice versa.

Pass `--target-dir` (or set `CARGO_TARGET_DIR`) to send the artifacts elsewhere.

Because those are settings the tool chooses, a check is a diagnostic pass rather than a reproduction of
your own `cargo check` — the same relationship Clippy has to your build.

## Flags are cargo's, not the tool's

The tool appends your arguments to `cargo check` and lets cargo own them. Three consequences are worth
knowing:

- Every `cargo check` flag behaves exactly as it does normally, because that is literally what runs.
- **cargo validates them, not cargo-cgp**, so an unknown flag produces cargo's error
  (`error: unexpected argument '--nope' found`) rather than a cargo-cgp message.
- `cargo cgp check --help` prints *cargo's* help and exits **without running a check**, since `--help`
  is forwarded like anything else.

The one flag the tool inspects is `--target-dir`, and only to decide whether to supply its own default.

## In your editor

Rust Analyzer can run the check on save. It is two words and must emit JSON, so wire it through
`check.overrideCommand` rather than `check.command`:

```json
"rust-analyzer.check.overrideCommand": [
  "cargo", "cgp", "check", "--workspace", "--all-targets", "--message-format=json"
]
```

The transformed diagnostics are rendered as rustc JSON, which cargo wraps and the editor parses, so the
rewritten messages appear inline. The isolated `target/cgp` directory earns its keep here too: the
editor's checking does not contend with your own builds.

## When it will not help

**A check only reports what your code asks the compiler to prove.** CGP's wiring is lazy, so a context
that is never checked and never used produces no error at all — from this tool or any other. Pair your
wiring with [`check_components!`](/docs/reference/macros/check_components), which is what forces a
missing dependency to surface at the wiring site instead of at some distant call.

**Some classes still pass through raw.** The tool is a `v0.1.0-alpha` that reshapes the core wiring
errors; orphan-rule violations, among others, arrive as the compiler wrote them.

If the command itself will not run, see [Troubleshooting](./troubleshooting.md).

---

*This page was written by an AI agent from the CGP knowledge base; its example and both outputs were
produced by running the tool — see
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
