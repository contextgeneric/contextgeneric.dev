---
sidebar_position: 4
---

# Expand

`cargo cgp expand` prints your crate as the compiler sees it after macro expansion, so you can read
what a CGP macro actually generated instead of reasoning about what it probably generated.

```sh
cargo cgp expand --lib
```

:::note

`expand` landed after the published `v0.1.0-alpha`, so an install from crates.io does not carry it yet.
Get it from the Nix flake with the tag dropped
(`nix run github:contextgeneric/cargo-cgp -- expand --lib`) or from a source build — see
[Installation](./installation.md#expand-is-newer-than-the-published-release). `cargo cgp check` is
unaffected.

:::

## What it looks like

Taking the `Rectangle` context from the [Check](./check.md) page, once its missing field is restored:

```sh
cargo cgp expand --lib --item Rectangle
```

```rust
pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}
impl HasField<Symbol!("width")> for Rectangle {
    type Value = f64;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("width")>,
    ) -> &Self::Value {
        &self.width
    }
}
impl HasFieldMut<Symbol!("width")> for Rectangle {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("width")>,
    ) -> &mut Self::Value {
        &mut self.width
    }
}
// … the same pair again for `height` …
impl DelegateComponent<AreaCalculatorComponent> for Rectangle {
    type Delegate = RectangleArea;
}
impl<
    __Context__,
    __Params__,
> IsProviderFor<AreaCalculatorComponent, __Context__, __Params__> for Rectangle
where
    RectangleArea: IsProviderFor<AreaCalculatorComponent, __Context__, __Params__>,
{}
impl __CheckRectangle<AreaCalculatorComponent, ()> for Rectangle {}
```

Everything a reader needs in order to follow a wiring error is visible there: the field accessors
`#[derive(HasField)]` produced, the one-line table entry `delegate_components!` produced, the
forwarding impl that carries the provider's requirements, and the assertion `check_components!` added.

## Narrowing with `--item`

A whole crate's expansion is long — for the tiny example above, 148 lines against 49 for the one item.
`--item` takes a `::`-separated path naming something *inside* the crate, and what you get depends on
what you name:

```sh
cargo cgp expand --lib --item shapes             # a module: its contents
cargo cgp expand --lib --item shapes::Rectangle  # a type: its declaration and every impl for it
cargo cgp expand --lib --item AreaCalculator     # a trait: its definition and every impl of it
```

A leading `crate::` is accepted, so `--item crate::contexts::app` and `--item contexts::app` are the
same request.

**The trait form is usually the one you want on CGP code**, because a component's generated items *are*
impls. Naming a component's *provider* trait gives you the provider trait, the delegation blanket impl,
the `UseContext` impl, and each wired provider's impl of it — the whole resolution path in one listing.
Naming the *consumer* trait gives just that trait and its routing blanket. The type form is the
companion, and is what the example above uses.

If the path matches nothing, the tool says so rather than falling back to a whole-crate dump:

```text
error: no module or item matched `NoSuchThing`
note: the path names a module or item inside the crate being expanded, as in `contexts::app`;
      a leading `crate::` is accepted too
```

## Choosing a target

`expand` expands exactly one target, and arguments are forwarded to `cargo rustc`, so target selection
is cargo's. A package with both a library and a binary therefore needs to be told which:

```sh
cargo cgp expand --lib
cargo cgp expand --bin my-app
cargo cgp expand -p my-crate --lib
```

Without one, cargo declines with *"extra arguments to `rustc` can only be passed to one target"*. That
is worth knowing before your first run, because the message is cargo's and does not mention `expand`.

Output goes to stdout, so it redirects and pipes like anything else:

```sh
cargo cgp expand --lib > expanded.rs
cargo cgp expand --lib | rg 'Symbol!'
```

`cargo cgp expand --help` forwards like everything else, so what you get is `cargo rustc`'s own help —
which does **not** mention `--item`, since that flag is the tool's rather than cargo's. This page is
where you learn it exists.

## Why not `cargo-expand`

`cargo-expand` is the general tool and it works on CGP code. The reason to reach for this one is that
CGP's generated code is dominated by *type-level constructs*, and a general expander prints them as the
compiler stores them.

**A field name is the clearest case.** CGP encodes it as a type-level string — one type parameter per
character. Expanded generically, the `height` tag above reads:

```text
Symbol<6, Chars<'h', Chars<'e', Chars<'i', Chars<'g', Chars<'h', Chars<'t', Nil>>>>>>>
```

`cargo cgp expand` resugars it back to `Symbol!("height")`, which is what you wrote. It does the same
for the rest of the vocabulary — a pipeline reads `Product![StepOne, StepTwo]` rather than a `Cons`
spine, and a namespace key reads `Path!(@app.GreeterComponent)`. On a page of wiring the difference is
between a listing you can read and one you have to decode.

Two smaller differences follow from the same purpose. **`--item` understands Rust items** — asking for
a trait gives you every impl *of* it, which is the shape a CGP question actually takes, rather than a
textual filter. And **the `cgp::macro_prelude::` qualifier is stripped**, which makes the output far
easier to read at the cost of not being compilable — see below.

## What to expect from the output

**Every macro is expanded, not only CGP's.** `#[derive(Debug)]` and `println!` appear in their
generated form too. The CGP-specific part is the resugaring, not the selection.

**The output is for reading, not compiling.** Stripping the `cgp::macro_prelude::` qualifier is what
makes it legible, and it also means you cannot paste the result back into your crate and expect it to
build.

**`expand` is not a check.** Compilation stops as soon as the macros are expanded, so no type analysis
runs and no CGP diagnostic is produced. A malformed macro invocation still fails, because that happens
during expansion — but a wiring mistake does not surface here at all. That is exactly what makes it
useful mid-debugging: it works on a crate that does not type-check. For the wiring mistake itself, use
[`cargo cgp check`](./check.md).

## When to reach for it

Four situations, all of them cases where the answer is in the generated code rather than in a message:

- **A diagnostic names a type you did not write** — `IsProviderFor`, a `PathCons` key, a `__Context__`
  parameter — and you want to see the impl it came from.
- **A wiring table does not resolve** and you want its real `DelegateComponent` keys and values rather
  than inferring them from the table's syntax.
- **You are unsure what a construct emits** — the exact `where` clauses of a provider impl, whether a
  getter's blanket impl needs the field you think it does.
- **Two forms differ and only one compiles.** Expand both and diff; the delta is the bug.

---

*This page was written by an AI agent from the CGP knowledge base; its example output was produced by
running the tool — see
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
