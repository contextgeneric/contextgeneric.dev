---
title: 'cargo-cgp error codes: every CGP-E code explained'
sidebar_label: 'Error codes'
sidebar_position: 7
description: 'Look up any [CGP-Exxx] code cargo cgp check prints: what the message says, what mistake it means, how to fix it, and where the class is explained.'
---

# Error codes

`cargo cgp check`, part of the [`cargo-cgp`](./index.md) toolchain for
[Context-Generic Programming (CGP)](/docs/), tags each CGP error it rewrites with a code in square
brackets, such as `[CGP-E001]`. This page lists every code: the message it heads, the mistake it means,
the fix, and the section of the [compile-errors reference](/docs/reference/errors) that explains the
class of mistake in full. [Reading the output](./reading-output.md) shows how the codes sit inside a
complete error.

## How the codes work

**A CGP code sits beside the compiler's own code, not in place of it.** In
`error[E0277]: [CGP-E001] …`, `E0277` is the compiler's code, which `rustc --explain` describes, and
`[CGP-E001]` is the CGP class the tool recognized. A rewritten error restates the compiler's error more
readably and keeps its code, so searching your output for a CGP code finds every error of that class.

The hundreds digit says where in the error a code appears:

- **`CGP-E0xx` codes head the error.** They classify the whole error: what failed.
- **`CGP-E1xx` codes label the dependency chain**, one per line of the tree beneath a `root cause:`
  note. The inner lines are steps along the way, and the last line is the cause.
- **`CGP-E2xx` codes label a `root cause:` line** whose cause has no `CGP-E1xx` code of its own.

In the messages below, a word in angle brackets, such as `<Context>`, stands for the name the tool fills
in from your code.

## Headline codes

### CGP-E001

**A context does not implement a consumer trait.** The context cannot use a component it is expected
to use: the component is not wired, or something the wired provider needs is missing, so the context
never gets the consumer trait.

```text
[CGP-E001] the consumer trait `<Consumer>` is not implemented for context `<Context>`
```

When one mistake breaks several components, the message names them all: "the consumer traits `<A>`,
`<B>`, and `<C>` are not implemented". The compiler's code stays whatever it assigned: `E0277` at a
check, `E0599` at a method call. **Fix:** follow the `root cause:` note, which names the missing field,
derive, wiring entry, or trait bound. Explained in
[A dependency is not met](/docs/reference/errors#a-dependency-is-not-met).

### CGP-E002

**A provider does not implement its provider trait for the context.** Something the provider requires
does not hold. This is the provider-side face of `[CGP-E001]`, and it is what a
`#[check_providers(...)]` assertion in
[`check_components!`](/docs/reference/macros/check_components) reports.

```text
[CGP-E002] the provider trait `<ProviderTrait>` with context `<Context>` is not implemented for provider `<Provider>`
```

**Fix:** follow the `root cause:` note to what the provider is missing. Explained in
[When the provider is a stack](/docs/reference/errors#when-the-provider-is-a-stack).

### CGP-E003

**A field is present but has the wrong type.** The context has the field and derives `HasField` for it,
but its type is not the one a provider reads.

```text
[CGP-E003] expected a `<field>` field of type `<expected>` on `<Context>`, but found `<actual>`
```

When the required type is written in terms of an abstract type, the message shows both forms, the
written one and what it resolves to:

```text
[CGP-E003] expected a `database` field of type `Pool<<App as HasDbType>::Db>` (`Pool<Postgres>`) on `App`, but found `Pool<Sqlite>`
```

The compiler's code is `E0271`. **Fix:** change the
field's type, or change the provider to accept the type the field has. Explained in
[When two decisions disagree about a type](/docs/reference/errors#when-two-decisions-disagree-about-a-type).

### CGP-E004

**The same key is wired twice.** A wiring table, or a namespace, maps one component or path to two
providers.

```text
[CGP-E004] duplicate wiring for <key> on `<Context>`
```

The compiler's two carets point at the two entries. **Fix:** remove one of them. Explained in
[Two entries claim one key](/docs/reference/errors#two-entries-claim-one-key).

### CGP-E005

**A key is wired that another entry already covers.** The two keys differ but overlap: a generic
entry over a specific one, a path that covers a longer path, or an entry for a key the context's
namespace already binds.

```text
[CGP-E005] `<Context>` cannot wire <key> that is already set through <source>
```

**Fix:** remove or narrow the overlapping entry. Explained in
[Two entries claim one key](/docs/reference/errors#two-entries-claim-one-key) and
[Overriding something a namespace already binds](/docs/reference/errors#overriding-something-a-namespace-already-binds).

### CGP-E006

**A context joins more than one namespace.** Each namespace covers every key, so two of them, or a
namespace and a loop over bare keys, conflict.

```text
[CGP-E006] only one namespace can be used for each target type in `delegate_components!`, but `<Context>` uses both `<A>` and `<B>`
```

**Fix:** join one namespace and make it inherit the others. Explained in
[Joining more than one namespace](/docs/reference/errors#joining-more-than-one-namespace).

### CGP-E007

**A direct entry collides with a redirect of the same key**, such as a component wired by name while an
`open` statement, a `=>` redirect, or a namespace sends it along a path.

```text
[CGP-E007] <component> on `<Context>` is redirected to `<path>`
```

A `help` line names the entry to write instead. **Fix:** wire the provider under the redirected path
rather than the component name. Explained in
[Overriding something a namespace already binds](/docs/reference/errors#overriding-something-a-namespace-already-binds).

### CGP-E008

**The same key is redirected twice**, by two `open` statements or `=>` mappings on one context, or by
one path registered twice in a namespace.

```text
[CGP-E008] duplicate redirect for <component> on `<Context>`
```

**Fix:** keep one redirect. Explained in
[Two entries claim one key](/docs/reference/errors#two-entries-claim-one-key).

### CGP-E009

**A trait that is not a CGP component fails because a component it depends on fails.** It is the
`[CGP-E001]` case for a trait you wrote by hand over a consumer trait, or for a
[`#[cgp_fn]`](/docs/reference/macros/cgp_fn) or [`#[blanket_trait]`](/docs/reference/macros/blanket_trait)
trait, which is why it says "the trait" rather than "the consumer trait".

```text
[CGP-E009] the trait `<Trait>` is not implemented for context `<Context>`
```

**Fix:** follow the `root cause:` note to the dependency the trait needs. Explained in
[A dependency is not met](/docs/reference/errors#a-dependency-is-not-met).

### CGP-E010

**The wiring for a component never resolves**, because looking it up leads back to itself. Almost
always a component is wired to `UseContext` on a context whose only implementation of it is that very
wiring.

```text
[CGP-E010] the wiring for the consumer trait `<Consumer>` on context `<Context>` never resolves — the lookup recurses without terminating
```

The compiler's code is `E0275`, and a `help` line names the usual cause. **Fix:** wire the component to
a real provider, or implement the consumer trait directly on the context. Explained in
[A lookup that never terminates](/docs/reference/errors#a-lookup-that-never-terminates).

### CGP-E011

**A crate registers wiring into a namespace it does not own, under a key it does not own either**,
which Rust's orphan rule forbids.

```text
[CGP-E011] cannot register the foreign <key> into the foreign namespace `<Namespace>`
```

The compiler's code is `E0210` or `E0117`. **Fix:** own one end, as the `help` line says: key the
registration on a component your crate defines, register it from the crate that owns the namespace, or
define a namespace of your own that inherits the foreign one. Explained in
[Registering into a namespace you do not own](/docs/reference/errors#registering-into-a-namespace-you-do-not-own).

### CGP-E012

**A `#[cgp_fn]` or `#[cgp_impl]` body calls a CGP trait's method on `self` without declaring the trait
as a dependency.** The macro turns the body into an implementation over a generated context type, so a
trait the body uses has to be declared for that type to have it.

```text
[CGP-E012] the trait `<Trait>` is used but not declared as a dependency
```

The compiler's code is `E0599`. **Fix:** add the trait to `#[uses(...)]`, as the `help` line says.
Explained in
[Using something you did not declare](/docs/reference/errors#using-something-you-did-not-declare).

### CGP-E013

**A `#[cgp_impl]` header names the component's consumer trait where its provider trait belongs.**

```text
[CGP-E013] `<Consumer>` is a consumer trait, but a `#[cgp_impl]` provider must implement its provider trait `<ProviderTrait>`
```

Without the rewrite, this one mistake produces a burst of unrelated-looking errors. **Fix:** name the
provider trait in the header, as the `help` line says. Explained in
[Naming the wrong half of a component](/docs/reference/errors#naming-the-wrong-half-of-a-component).

### CGP-E014

**`#[cgp_impl]` is applied to a trait that is not a CGP component at all**, so there is no provider
trait to implement.

```text
[CGP-E014] `#[cgp_impl]` can only implement a CGP component's provider trait, but `<Trait>` is not a CGP component
```

**Fix:** make the trait a component with
[`#[cgp_component]`](/docs/reference/macros/cgp_component), or drop `#[cgp_impl]` and write an ordinary
`impl`. Explained in
[Naming the wrong half of a component](/docs/reference/errors#naming-the-wrong-half-of-a-component).

### CGP-E015

**An inner-provider bound names a consumer trait where the provider trait belongs**, most often in
[`#[use_provider]`](/docs/reference/attributes/use_provider).

```text
[CGP-E015] `<Consumer>` is a consumer trait and cannot bound an inner provider; a higher-order provider imports its provider trait `<ProviderTrait>`
```

**Fix:** name the provider trait, as the `help` line says. Explained in
[Naming the wrong half of a component](/docs/reference/errors#naming-the-wrong-half-of-a-component).

### CGP-E016

**A higher-order provider calls an inner provider it never imported**, so the inner provider has no
provider-trait bound and the call cannot resolve.

```text
[CGP-E016] the inner provider `<Inner>` is used but not imported
```

**Fix:** import it with `#[use_provider(<Inner>: <ProviderTrait>)]`, as the `help` line says. Explained
in [Using something you did not declare](/docs/reference/errors#using-something-you-did-not-declare).

### CGP-E017

**An abstract type is wired one way and required another.** The context supplies one type for an
associated type, such as its error type, while a provider requires a different one.

```text
[CGP-E017] expected the abstract type `<Type>` of `<Trait>` on `<Owner>` to be `<expected>`, but found `<actual>`
```

For a trait that is not a CGP abstract-type component, the message says "associated type" instead. Like
`[CGP-E003]`, it shows a required type in both forms when it is written through another abstract type.
The compiler's code is `E0271`. **Fix:** make the two sides agree; for a
[`#[cgp_type]`](/docs/reference/macros/cgp_type) component, the `help` line names the `UseType` entry to
wire. Explained in
[When two decisions disagree about a type](/docs/reference/errors#when-two-decisions-disagree-about-a-type).

## Dependency-chain codes

These codes label the lines of the tree beneath a `root cause:` note. The first four are steps along
the way; the rest end the chain, and the cause they state is the one to fix.

### CGP-E101

**A step through a context's consumer trait.** Usually the first line of the chain.

```text
[CGP-E101] consumer trait impl `<Consumer>` for context `<Context>`
```

### CGP-E102

**A step through a provider's provider trait**: the context's wiring selected this provider, and the
chain continues with what the provider needs.

```text
[CGP-E102] provider trait impl `<ProviderTrait>` with context `<Context>` for provider `<Provider>`
```

### CGP-E104

**A step through a redirected lookup**, from an `open` statement or a namespace, naming the table the
path is looked up in.

```text
[CGP-E104] redirect lookup to `@<path>` in `<Table>`
```

### CGP-E105

**A step through any other trait**, such as a getter or a blanket trait you wrote.

```text
[CGP-E105] trait impl `<Trait>` for `<Type>`
```

`CGP-E103` is not assigned.

### CGP-E106

**The chain ends at a field the context does not have.**

```text
[CGP-E106] missing field `<field>` on `<Type>`
```

**Fix:** add the field, or wire a provider that does not read it. Explained in
[A dependency is not met](/docs/reference/errors#a-dependency-is-not-met).

### CGP-E107

**The chain ends at a component or path the context wires nothing for.**

```text
[CGP-E107] context `<Context>` does not contain any delegate entry for `<key>`
```

**Fix:** add the wiring entry. Explained in
[Nothing is wired there](/docs/reference/errors#nothing-is-wired-there).

### CGP-E108

**The chain ends at a field that exists but has no `HasField` impl**, because the struct does not derive
it.

```text
[CGP-E108] accessor trait `HasField` with field `<field>` is not implemented for `<Type>`
```

**Fix:** add `#[derive(HasField)]` to the struct, as the `help` line says. Explained in
[When the derive is missing entirely](/docs/reference/errors#when-the-derive-is-missing-entirely).

### CGP-E109

**The chain ends at a field with the wrong type.** It is the last line of a `[CGP-E003]` error.

```text
[CGP-E109] field `<field>` on `<Type>` has type `<actual>`, but `<expected>` is required
```

### CGP-E110

**The chain ends at a provider's own table that has no entry for a key**: a bundle of wiring missing a
component, or a provider that dispatches per type missing the type it was given.

```text
[CGP-E110] provider `<Provider>` does not contain any delegate entry for `<key>`
```

**Fix:** add the entry to that provider's table, or pass it a type the table covers.

### CGP-E111

**The chain ends at a type wired where a provider belongs that is not a provider**, such as a value
type put in a provider's place.

```text
[CGP-E111] the provider trait `<ProviderTrait>` is not implemented for `<Type>`
```

**Fix:** wire an actual provider in that position.

### CGP-E112

**The chain ends at an associated type that differs from what was required.** It is the last line of a
`[CGP-E017]` error.

```text
[CGP-E112] abstract type `<Type>` of `<Trait>` on `<Owner>` is `<actual>`, but `<expected>` is required
```

## Root-cause codes

A `root cause:` line normally carries the same code as the last line of its chain. This range covers
the case where that last line has no code.

### CGP-E201

**The chain ends at an ordinary Rust trait bound that does not hold**, such as `f64: Eq`. The last line
of the tree repeats the compiler's own wording without a code, and the `root cause:` line carries this
one. The line appears when the error has a CGP headline, as at a method call; at a
`check_components!` entry the tool keeps the compiler's own headline, which already names the bound,
and adds the chain without a `root cause:` line.

```text
root cause: [CGP-E201] the trait bound `<Type: Trait>` is not satisfied
```

**Fix:** use a type that implements the trait, or a provider that does not require it. Explained in
[When the missing bound is an ordinary Rust trait](/docs/reference/errors#when-the-missing-bound-is-an-ordinary-rust-trait).

## Rewrites without a code

Some changes make an error easier to read without classifying it, so they carry no code. You may see
them in an error whose headline the tool left as the compiler wrote it:

- **Type-level names written back as you wrote them**: a field name as `Symbol!("name")`, a list as
  `Product![A, B]` or `Sum![A, B]`, and a path as `Path!(@app.Component)`. A path that ends in an
  unknown part gets a trailing `.*`, which is not real `Path!` syntax.
- **Shorter type names**, with the CGP module paths the compiler prints in front of them removed.
- **A missing-field sentence** in place of an unmet `HasField` requirement inside a note.
- **Consumer and provider trait names** in the compiler's "required for … to implement …" notes, in
  place of the generated traits that connect them.
- **Removed advice** that a method call should use associated-function syntax, which the compiler
  suggests for CGP's provider methods and which does not fix the mistake.

---

*An AI agent wrote this page using the CGP knowledge base. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
