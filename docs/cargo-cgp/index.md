---
sidebar_position: 1
---

# cargo-cgp

`cargo-cgp` is CGP's error toolchain: a cargo subcommand that stands in for `cargo check` and presents
CGP's compiler errors root cause first, plus a second command that shows the ordinary Rust your macros
generate.

## The problem it solves

CGP's wiring is **lazy**. A `delegate_components!` entry is accepted without checking that the provider
it names can actually do the job, so a context with a missing field or an unmet dependency compiles
fine and fails somewhere else entirely — usually at the first call to the capability, in a message
about types you never wrote.

Here is the whole of a mistake: a `Rectangle` that lost its `height` field while a provider still reads
it.

```rust
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
```

Under plain `cargo check`, with the mistake pinned to the wiring site by
[`check_components!`](/docs/reference/macros/check_components), this is what comes back:

```text
error[E0277]: the trait bound `Rectangle: HasField<Symbol<6, Chars<'h', ...>>>` is not satisfied
   |
28 |         AreaCalculatorComponent,
   |         ^^^^^^^^^^^^^^^^^^^^^^^ unsatisfied trait bound
   |
help: the trait `HasField<Symbol<6, cgp::prelude::Chars<'h', cgp::prelude::Chars<'e',
      cgp::prelude::Chars<'i', cgp::prelude::Chars<'g', cgp::prelude::Chars<_,
      cgp::prelude::Chars<'t', Nil>>>>>>>>` is not implemented for `Rectangle`
```

The answer is in there — the field is spelled out one character per type parameter — but you have to
decode a `Chars` list to read it, and this is the *good* case, where a check forced the failure to the
wiring site.

`cargo cgp check` reports the same mistake like this:

```text
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
```

Same error, same `E0277`, same span. What changed is that the cause is named in English on the second
line, and the chain that demanded it is drawn out beneath.

## How it does that

Two things, and the first matters more than the reshaping.

**It un-hides causes the compiler suppresses.** The worst class of CGP error is not verbose but
*incomplete*: the default trait solver stops at "the trait bounds were not satisfied" without ever
naming the bound that failed, so the answer is absent from the output rather than buried in it.
`cargo cgp check` compiles under Rust's next-generation trait solver, which reports the real missing
bound — and that is what makes a root cause available to print at all.

**It rewrites the classes it recognizes.** Where the tool knows the shape of a CGP failure, it leads
with the cause, draws the dependency chain as a tree, and stamps the message with a `[CGP-Exxx]` code
naming the class. The compiler's own code is always kept, so `rustc --explain E0277` still works and
nothing is reclassified away from rustc.

## The two commands

[**`cargo cgp check`**](./check.md) stands in for `cargo check` when you hit or expect a wiring error.
Everything after `check` is forwarded to cargo, so the flags you already use work unchanged.

[**`cargo cgp expand`**](./expand.md) prints what your CGP macros generated, with CGP's type-level
constructs spelled the way you wrote them — a field tag reads `Symbol!("height")` rather than the
six-level `Chars` list above. It is how you answer "what did that macro actually produce?" rather than
reasoning about what it probably produced.

## What it is not

**It is optional, and it is not a build tool.** CGP is an ordinary library that compiles on **stable
Rust 1.89 or newer**, so `cargo build`, `cargo run`, and `cargo test` work on a CGP project unchanged
and should stay as they are. There is no `cargo cgp build` or `cargo cgp test`. Reach for
`cargo cgp check` when you want a wiring error made readable, and plain `cargo check` when you do not.

**It is a diagnostic pass rather than a reproduction of your own build.** A check runs under the tool's
pinned nightly, turns on the next-generation solver, and builds into a separate `target/cgp` directory,
so it neither adopts your project's toolchain nor disturbs its build cache — much as Clippy runs under
its own settings.

**It is a `v0.1.0-alpha`, and the honest description is *dramatically better, not solved*.** The tool
reshapes the core wiring errors and passes everything else through as the compiler wrote it, so some
classes — orphan-rule violations among them — still arrive raw. If you hit one, the
[reference](/docs/reference) and [`check_components!`](/docs/reference/macros/check_components) are
still the tools that localize it.

## Next

- [Installation](./installation.md) — cargo, Nix, running it without installing, or asking your coding
  assistant to set it up.
- [Check](./check.md) — running it, reading the output, and wiring it into your editor.
- [Expand](./expand.md) — seeing through the macros, and why it is not `cargo-expand`.
- [Troubleshooting](./troubleshooting.md) — matching an error to its cause when the tool itself will not
  run.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the tool's own
output — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
