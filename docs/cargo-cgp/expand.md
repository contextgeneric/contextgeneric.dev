---
title: 'cargo cgp expand: see what CGP macros generate'
sidebar_label: 'Expand'
sidebar_position: 5
description: 'Print the Rust your CGP macros generate, narrowed to one module, type, or trait, with field names and type-level lists written the way you wrote them.'
---

# Expand

`cargo cgp expand` prints your crate as the compiler sees it after macro expansion, so you can read what
a [CGP](/docs/) macro actually generated instead of reasoning about what it probably generated. It is
part of [`cargo-cgp`](./index.md), and it writes CGP's type-level constructs back in the form you wrote
them.

```sh
cargo cgp expand --lib
```

## When to use it

Reach for `expand` when the answer is in the generated code rather than in an error message:

- **A diagnostic names a type you did not write**, such as
  [`IsProviderFor`](/docs/reference/traits/wiring/is_provider_for), a `PathCons` key, or a
  `__Context__` parameter, and you want to see the impl it came from.
- **A wiring table does not resolve**, and you want its actual
  [`DelegateComponent`](/docs/reference/traits/wiring/delegate_component) keys and providers rather
  than inferring them from the table's syntax.
- **You are unsure what a construct emits**, such as the exact `where` clauses of a provider, or which
  field a getter reads.
- **Two forms differ and only one compiles.** Expand both and compare the output.

## What it looks like

This is the `Rectangle` program from the [Check page](./check.md#check-the-wiring-where-you-write-it)
with its `height` field restored, so the program compiles:

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
    pub height: f64,
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

Asking for the `Rectangle` type gives its declaration and every impl written for it:

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
impl HasField<Symbol!("height")> for Rectangle {
    type Value = f64;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("height")>,
    ) -> &Self::Value {
        &self.height
    }
}
impl HasFieldMut<Symbol!("height")> for Rectangle {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("height")>,
    ) -> &mut Self::Value {
        &mut self.height
    }
}
impl DelegateComponent<AreaCalculatorComponent> for Rectangle {
    type Delegate = RectangleArea;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<AreaCalculatorComponent, __Context__, __Params__> for Rectangle
where
    RectangleArea: IsProviderFor<AreaCalculatorComponent, __Context__, __Params__>,
{}
impl __CheckRectangle<AreaCalculatorComponent, ()> for Rectangle {}
```

Each part of the program is visible in the listing:

- [`#[derive(HasField)]`](/docs/reference/derives/derive_has_field) produced a pair of field accessors
  for each field, keyed by the field's name.
- [`delegate_components!`](/docs/reference/macros/delegate_components) produced the one-line table
  entry that selects `RectangleArea`, and an impl that passes the provider's requirements through to
  `Rectangle`.
- [`check_components!`](/docs/reference/macros/check_components) produced the last line, the
  assertion that `Rectangle` can use the component.

## Narrowing with `--item`

**`--item` narrows the output to one module or item.** A whole crate's expansion is long: for this
small program it is 148 lines, against 49 for `Rectangle` alone. The option takes a `::`-separated
path naming something inside the crate, and what you get depends on what it names:

```sh
cargo cgp expand --lib --item shapes             # a module: its contents
cargo cgp expand --lib --item shapes::Rectangle  # a type: its declaration and every impl for it
cargo cgp expand --lib --item AreaCalculator     # a trait: its definition and every impl of it
```

A leading `crate::` is accepted, so `--item crate::shapes` and `--item shapes` are the same request.

**Naming a component's provider trait is usually the most useful form**, because most of a
component's generated items are impls of it. For this program, `--item AreaCalculator` gives the
provider trait itself, the blanket impl that lets a context's wiring pick the provider, the impls for
the `UseContext` and `RedirectLookup` providers CGP generates for every component, and
`RectangleArea`'s own impl, with the `where` clause that requires the two fields. Naming the consumer
trait instead, as in `--item CanCalculateArea`, gives that trait and the one blanket impl that connects
it to the provider trait.

If the path matches nothing, the tool says so instead of printing the whole crate. Among cargo's
progress lines, it prints:

```text
error: no module or item matched `NoSuchThing`
note: the path names a module or item inside the crate being expanded, as in `contexts::app`; a leading `crate::` is accepted too
```

and finishes with:

```text
error: no expansion was produced; see the errors above
```

## Choosing a target

`expand` expands exactly one target, and its other arguments are passed to `cargo rustc`, so choosing
the target is cargo's job. A package with both a library and a binary needs to be told which one:

```sh
cargo cgp expand --lib
cargo cgp expand --bin my-app
cargo cgp expand -p my-crate --lib
```

Without one, cargo declines, in a message that does not mention `expand`:

```text
error: extra arguments to `rustc` can only be passed to one target, consider filtering
the package by passing, e.g., `--lib` or `--bin NAME` to specify a single target
error: no expansion was produced; see the errors above
```

The expansion goes to standard output, so it redirects and pipes like any other program's output:

```sh
cargo cgp expand --lib > expanded.rs
cargo cgp expand --lib | grep 'Symbol!'
```

`cargo cgp expand --help` prints the tool's own help for `expand`, which documents `--item`. That is
the one argument `expand` reads itself; for cargo's target-selection options, run `cargo rustc --help`.

## Why not `cargo-expand`

[`cargo-expand`](https://github.com/dtolnay/cargo-expand) is the general tool, and it works on CGP code.
The reason to use this one is that CGP's generated code is full of type-level constructs, which a
general expander prints the way the compiler stores them.

**A field name is the clearest case.** CGP encodes it as a
[type-level string](/docs/reference/glossary#type-level-string), with one type per character. In the
compiler's own expansion, which is what a general expander formats, the `height` accessor above begins:

```text
impl ::cgp::macro_prelude::HasField<::cgp::macro_prelude::Symbol<6,
    ::cgp::macro_prelude::Chars<'h',
    ::cgp::macro_prelude::Chars<'e',
    ::cgp::macro_prelude::Chars<'i',
    ::cgp::macro_prelude::Chars<'g',
    ::cgp::macro_prelude::Chars<'h',
    ::cgp::macro_prelude::Chars<'t', ::cgp::macro_prelude::Nil>>>>>>>> for
    Rectangle {
```

`cargo cgp expand` writes it back as `Symbol!("height")`, which is what you wrote. It does the same for
the rest of CGP's type-level vocabulary: a list of handlers reads `Product![StepOne, StepTwo]` rather
than a chain of `Cons` cells, and a namespace key reads `Path!(@app.GreeterComponent)`.

`--item` is the other difference. It selects Rust items rather than filtering text: asking for a trait
gives every impl of that trait, wherever it sits in the crate, which is the shape a question about CGP
wiring usually takes.

## What to expect from the output

**Every macro is expanded, not only CGP's.** `#[derive(Debug)]` and `println!` appear in their generated
form too. What is specific to CGP is how the type-level constructs are written.

**The output is for reading, not compiling.** The tool removes the `cgp::macro_prelude::` prefix the
macros put in front of every CGP name, which makes the listing readable and means it does not compile
if you paste it back into your crate.

**`expand` is not a check.** Compilation stops once the macros are expanded, so no type checking runs
and no wiring error is reported. A malformed macro invocation still fails, because that happens during
expansion. This is what makes `expand` useful in the middle of debugging: it works on a crate that does
not type-check. For the wiring mistake itself, use [`cargo cgp check`](./check.md).

**It builds the way a check does.** `expand` uses the same pinned nightly and the same `target/cgp`
directory as `cargo cgp check`, so the notes on the [Check page](./check.md#what-a-check-does-differently)
apply to it too.

---

*An AI agent wrote this page using the CGP knowledge base, and its outputs were produced by running the
commands. See [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
