---
title: 'cargo-cgp error codes: every CGP-E code explained'
sidebar_label: 'Error codes'
sidebar_position: 8
description: 'Look up any [CGP-Exxx] code cargo cgp check prints: what the message says, what mistake it means, how to fix it, and where the class is explained.'
---

# Error codes

When `cargo cgp check` rewrites a CGP error, it tags the message with a code in square brackets, such as
`[CGP-E001]`, so you can look up what the message means. The command is part of the
[`cargo-cgp`](./index.md) toolchain for [Context-Generic Programming (CGP)](/docs/). This page lists every
code with the message it heads, the mistake it means, the fix, and the section that explains the class of
mistake in full, usually in the [compile-errors reference](/docs/reference/errors).
[Reading the output](./reading-output.md) shows how the codes sit inside a complete error.

## How the codes work

A CGP code sits beside the compiler's own code rather than in place of it. In
`error[E0277]: [CGP-E001] …`, `E0277` is the compiler's code, which `rustc --explain` describes, and
`[CGP-E001]` is the CGP class the tool recognized. Because a rewritten error restates the compiler's error
and keeps its code, searching your output for a CGP code finds every error of that class.

The hundreds digit says where in the error a code appears. `CGP-E0xx` codes head the error and classify it
as a whole. `CGP-E1xx` codes label the dependency chain, one per line of the tree beneath a `root cause:`
note. From `CGP-E101` to `CGP-E104` they mark steps along the way, and from `CGP-E110` onward they mark the
cause the chain ends at.

In the messages below, a word in angle brackets, such as `<Context>`, stands for the name the tool fills in
from your code.

## A trait the context cannot use

The first family covers a requirement that fails somewhere under a trait. The tool traces the failure to its
causes, so the error usually carries a `root cause:` note and a dependency chain.

### CGP-E001

A context cannot use a component it is expected to use. Either the component is not wired, or something the
wired provider needs is missing, so the context never gets the consumer trait. The headline states the cause
in the words of its chain line:

```text
[CGP-E001] missing field `height` on `Rectangle`
```

Causes of one kind on one owner are stated together, as in ``missing fields `width` and `height` on
`Rectangle` ``. A list of more than four names the first three and counts the rest. Causes of different kinds
get the class sentence with their count instead:

```text
[CGP-E001] the consumer trait `<Consumer>` is not implemented for context `<Context>`: 2 root causes
```

The compiler's code stays whatever it assigned, which is `E0277` at a check and `E0599` at a method call. To
fix it, follow the `root cause:` note, which names the missing field, derive, wiring entry, or trait bound.
[A dependency is not met](/docs/reference/errors#a-dependency-is-not-met) explains the class.

### CGP-E002

A provider does not implement its provider trait for the context, because something the provider requires
does not hold. This is the provider-side version of `[CGP-E001]`, and a failing `#[check_providers(...)]`
assertion in [`check_components!`](/docs/reference/macros/check_components) or a call through the provider's
own path reports it. It also states the cause in its headline, with this class sentence for mixed causes:

```text
[CGP-E002] the provider trait `<ProviderTrait>` with context `<Context>` is not implemented for provider `<Provider>`
```

To fix it, follow the `root cause:` note to what the provider is missing.
[When the provider is a stack](/docs/reference/errors#when-the-provider-is-a-stack) explains the class.

### CGP-E003

A trait that is not a CGP component fails because a component it depends on fails. It is the `[CGP-E001]` case
for a trait you wrote by hand over a consumer trait, or for a [`#[cgp_fn]`](/docs/reference/macros/cgp_fn) or
[`#[blanket_trait]`](/docs/reference/macros/blanket_trait) trait, so its class sentence says "the trait" rather
than "the consumer trait":

```text
[CGP-E003] the trait `<Trait>` is not implemented for context `<Context>`
```

To fix it, follow the `root cause:` note to the dependency the trait needs.
[A dependency is not met](/docs/reference/errors#a-dependency-is-not-met) explains the class.

### CGP-E004

A field is present but has the wrong type. The context has the field and derives `HasField` for it, but its type
is not the one a provider reads:

```text
[CGP-E004] field `<field>` on `<Context>` has type `<actual>`, but `<expected>` is required
```

When the required type is written in terms of an abstract type, the message shows both the written form and what
it resolves to:

```text
[CGP-E004] field `database` on `App` has type `Pool<Sqlite>`, but `Pool<<App as HasDbType>::Db>` (`Pool<Postgres>`) is required
```

The compiler's code is `E0271`. To fix it, change the field's type, or change the provider to accept the type the
field has.
[When two decisions disagree about a type](/docs/reference/errors#when-two-decisions-disagree-about-a-type)
explains the class.

### CGP-E005

An abstract type is wired one way and required another. The context supplies one type for an associated type,
such as its error type, while a provider requires a different one:

```text
[CGP-E005] abstract type `<Type>` of `<Trait>` on `<Owner>` is `<actual>`, but `<expected>` is required
```

For a trait that is not a CGP abstract-type component, the message says "associated type" instead. Like
`[CGP-E004]`, it shows a required type in both forms when it is written through another abstract type. The
compiler's code is `E0271`. To fix it, make the two sides agree. For a
[`#[cgp_type]`](/docs/reference/macros/cgp_type) component, the `help` line names the `UseType` entry to wire.
[When two decisions disagree about a type](/docs/reference/errors#when-two-decisions-disagree-about-a-type)
explains the class.

## Conflicting wiring

These five codes rewrite an `E0119` on a wiring entry, and the compiler's two carets point at the entries that
collide. A `<key>` is worded as what it is, such as `` component `<Marker>` ``, `` type `String` ``, or `` `@a.b` ``.
[Two entries claim one key](/docs/reference/errors#two-entries-claim-one-key) explains the class.

### CGP-E010

The same key is wired twice, because a wiring table or a namespace maps one component or path to two providers. To
fix it, remove one of the entries.

```text
[CGP-E010] conflicting wiring: `<Context>` wires <key> twice
```

### CGP-E011

Two different keys overlap, so an entry is wired that another already covers. It may be a generic entry over a
specific one, a path that covers a longer path, or an entry for a key the context's namespace already binds. To fix
it, remove or narrow the overlapping entry.
[Overriding something a namespace already binds](/docs/reference/errors#overriding-something-a-namespace-already-binds)
covers the namespace case.

```text
[CGP-E011] conflicting wiring: `<Context>` wires <key>, which <source> already covers
```

The source is the key that covers the other, whichever comes first. When neither covers the other, the key is the
one they share.

### CGP-E012

A context joins more than one namespace. Each namespace covers every key, so two of them, or a namespace and a loop
over bare keys, conflict. To fix it, join one namespace and make it inherit the others.
[Joining more than one namespace](/docs/reference/errors#joining-more-than-one-namespace) explains the class.

```text
[CGP-E012] conflicting wiring: `<Context>` joins both `<A>` and `<B>`, but a target type in `delegate_components!` can join only one namespace
```

### CGP-E013

A direct entry collides with a redirect of the same key, such as a component wired by name while an `open`
statement, a `=>` redirect, or a namespace sends it along a path. To fix it, wire the provider under the redirected
path rather than the component name, as the `help` line says.
[Overriding something a namespace already binds](/docs/reference/errors#overriding-something-a-namespace-already-binds)
explains the class.

```text
[CGP-E013] conflicting wiring: `<Context>` wires <component> directly, but it is redirected to `<path>`
```

### CGP-E014

The same key is redirected twice, by two `open` statements or `=>` mappings on one context, or by one path registered
twice in a namespace. To fix it, keep one redirect.

```text
[CGP-E014] conflicting wiring: `<Context>` redirects <component> twice, to `<path>` and to `<path>`
```

## Structural mistakes

These codes name a mistake in the shape of what a macro was given, which the compiler rejects through generated items.
Each one uses your program's terms, drops the compiler's suggestions that do not apply, and gives the fix in a `help`
line.

### CGP-E015

A `check_components!` table lists one entry twice. To fix it, remove one of the two.
[The wiring itself does not compile](/docs/reference/errors#the-wiring-itself-does-not-compile) covers this family.

```text
[CGP-E015] `check_components!` checks `<entry>` twice for `<Context>`
```

### CGP-E016

Two `check_components!` tables in one module check the same context, so they share a check-trait name. To fix it, merge
the tables, or name one table's check trait with `#[check_trait(...)]`. When two attributes give one name, the message
says so instead.
[The wiring itself does not compile](/docs/reference/errors#the-wiring-itself-does-not-compile) covers this family.

```text
[CGP-E016] two `check_components!` tables in one module check `<Context>`
```

### CGP-E017

A `for` loop in the wiring binds a variable its key does not use. To fix it, write the variable in the key where the
type the loop covers goes.
[A generic that reaches nothing](/docs/reference/errors#a-generic-that-reaches-nothing) explains the class.

```text
[CGP-E017] the `for` loop in the wiring of `<Context>` binds `<T>`, but its key `<key>` does not use it
```

### CGP-E018

A `#[default_impl]` registers a provider whose impl is generic over a parameter the key does not fix. To fix it, write
the impl for that key alone, as the `help` line says.
[A generic that reaches nothing](/docs/reference/errors#a-generic-that-reaches-nothing) explains the class.

```text
[CGP-E018] `#[default_impl]` cannot register `<Provider>` for `<key>`, because the provider's impl is generic over `<T>`
```

### CGP-E019

A wiring or namespace entry binds a generic parameter its key does not use. To fix it, write a concrete type in place of
the parameter and remove it from the entry's generic parameters, as the `help` line says.
[A generic that reaches nothing](/docs/reference/errors#a-generic-that-reaches-nothing) explains the class.

```text
[CGP-E019] the entry for `<key>` in the wiring of `<Context>` binds `<T>`, but its key does not use it
```

### CGP-E060

A check entry names no parameters for a component that takes some, so it checks the component at the unit parameters
`()`, which the provider does not implement. To fix it, check it at the parameters the `help` line offers, with
`#[check_params(...)]` in `delegate_and_check_components!` or `<entry>: <P>` in `check_components!`.
[`check_components!`](/docs/reference/macros/check_components) describes the syntax.

```text
[CGP-E060] the check of `<entry>` on `<Context>` names no parameters, so it checks the component at the unit parameters `()`
```

### CGP-E061

Two [`#[cgp_auto_dispatch]`](/docs/reference/macros/cgp_auto_dispatch) traits in one module declare a method of the same
name, and the macro names its generated items after the method alone. To fix it, rename the method in one trait, or
declare the traits in different modules.

```text
[CGP-E061] two `#[cgp_auto_dispatch]` traits in one module declare the method `<method>`
```

## Wiring that never resolves, and the orphan rule

The next three codes cover wiring that is well formed but cannot work, either because looking it up never ends or because
Rust's coherence rules forbid it.

### CGP-E020

Looking up the wiring for a component leads back to itself. The plain case is a component wired to `UseContext` on a
context with no other implementation of it. The compiler's code is `E0275`. To fix it, wire a real provider, or implement
the consumer trait directly on the context. The `help` line names the provider that closes the loop.
[A lookup that never terminates](/docs/reference/errors#a-lookup-that-never-terminates) explains the class.

```text
[CGP-E020] the wiring for the consumer trait `<Consumer>` on context `<Context>` never resolves, because the lookup leads back to itself
```

### CGP-E021

Each step of the lookup requires the consumer trait again for a larger type, so nothing repeats, and the compiler stops
at its recursion limit. No limit ends a lookup that grows, so to fix it, wire a provider whose requirements do not grow.
[A lookup that never terminates](/docs/reference/errors#a-lookup-that-never-terminates) explains the class.

```text
[CGP-E021] the wiring for the consumer trait `<Consumer>` on context `<Context>` never resolves, because each step requires it for a larger type (`<Larger>`, …)
```

### CGP-E022

A crate registers wiring into a namespace it does not own, under a key it does not own either, which Rust's orphan rule
forbids. The compiler's code is `E0210` or `E0117`. To fix it, own one end, as the `help` line says: key the registration
on a component your crate defines, register it from the crate that owns the namespace, or define a namespace of your own
that inherits the foreign one.
[Registering into a namespace you do not own](/docs/reference/errors#registering-into-a-namespace-you-do-not-own)
explains the class.

```text
[CGP-E022] cannot register the foreign <key> into the foreign namespace `<Namespace>`
```

## Mistakes in what a macro was given

These codes name a CGP attribute or macro used in a way the generated code cannot compile. Most of them give the fix in a
`help` line.

### CGP-E030

A `#[cgp_fn]` or `#[cgp_impl]` body calls a CGP trait's method on `self` or `Self` without declaring the trait as a
dependency. The macro turns the body into an implementation over a generated context type, so a trait the body uses has to
be declared for that type to have it. The compiler's code is `E0599`. To fix it, add the trait to `#[uses(...)]`, as the
`help` line says.
[Using something you did not declare](/docs/reference/errors#using-something-you-did-not-declare) explains the class.

```text
[CGP-E030] the trait `<Trait>` is used but not declared as a dependency
```

### CGP-E031

A higher-order provider calls an inner provider it never imported, so the inner provider has no provider-trait bound and
the call cannot resolve. To fix it, import it with `#[use_provider(<Inner>: <ProviderTrait>)]`, as the `help` line says.
[Using something you did not declare](/docs/reference/errors#using-something-you-did-not-declare) explains the class.

```text
[CGP-E031] the inner provider `<Inner>` is used but not imported
```

### CGP-E032

A `#[cgp_impl]` header names the component's consumer trait where its provider trait belongs. Without the rewrite, this one
mistake produces a burst of unrelated-looking errors. To fix it, name the provider trait in the header, as the `help` line
says. [Naming the wrong half of a component](/docs/reference/errors#naming-the-wrong-half-of-a-component) explains the
class.

```text
[CGP-E032] `<Consumer>` is a consumer trait, but a `#[cgp_impl]` provider must implement its provider trait `<ProviderTrait>`
```

### CGP-E033

`#[cgp_impl]` is applied to a trait that is not a CGP component at all, so there is no provider trait to implement. To fix
it, make the trait a component with [`#[cgp_component]`](/docs/reference/macros/cgp_component) and implement its provider
trait, or drop `#[cgp_impl]` and write an ordinary `impl`.
[Naming the wrong half of a component](/docs/reference/errors#naming-the-wrong-half-of-a-component) explains the class.

```text
[CGP-E033] `#[cgp_impl]` can only implement a CGP component's provider trait, but `<Trait>` is not a CGP component
```

### CGP-E034

An inner-provider bound names a consumer trait where the provider trait belongs, most often in
[`#[use_provider]`](/docs/reference/attributes/use_provider). To fix it, name the provider trait, as the `help` line says.
[Naming the wrong half of a component](/docs/reference/errors#naming-the-wrong-half-of-a-component) explains the class.

```text
[CGP-E034] `<Consumer>` is a consumer trait and cannot bound an inner provider; a higher-order provider imports its provider trait `<ProviderTrait>`
```

### CGP-E035

A `#[cgp_fn]`, `#[cgp_impl]`, or `#[cgp_component]` definition names an abstract type of the context, as `Self::Error` or a
bare `Error`, without [`#[use_type]`](/docs/reference/attributes/use_type). To fix it, import the type with
`#[use_type(<Trait>.<Type>)]`, as the `help` line says, and write it bare.
[Using something you did not declare](/docs/reference/errors#using-something-you-did-not-declare) explains the class.

```text
[CGP-E035] the abstract type `<Type>` is used but not imported
```

### CGP-E036

A component requires a trait of every context, through [`#[extend(...)]`](/docs/reference/attributes/extend) or
`#[use_type(...)]`, and a `#[cgp_impl]` provider does not declare it, even if its body never uses the trait. To fix it, add
the `#[uses(...)]` or `#[use_type(...)]` the `help` line names.
[Using something you did not declare](/docs/reference/errors#using-something-you-did-not-declare) explains the class.

```text
[CGP-E036] the provider trait `<ProviderTrait>` requires `<Trait>` of the context, but the provider `<Provider>` does not declare it
```

### CGP-E037

[`#[uses]`](/docs/reference/attributes/uses) names a provider trait, but a dependency names its consumer trait. To fix it,
name the consumer trait, as the `help` line says.
[Naming the wrong half of a component](/docs/reference/errors#naming-the-wrong-half-of-a-component) explains the class.

```text
[CGP-E037] `<ProviderTrait>` is a provider trait and cannot be a `#[uses]` dependency; a dependency names its consumer trait `<Consumer>`
```

### CGP-E038

`#[cgp_impl(GreetHello)]` without `new` implements the provider trait for a struct nothing declares. To fix it, write
`#[cgp_impl(new GreetHello)]` so the macro declares it, or declare the struct yourself. A provider declared elsewhere but not
imported keeps the compiler's `E0425` and its import suggestion instead.
[`#[cgp_impl]`](/docs/reference/macros/cgp_impl) describes the syntax.

```text
[CGP-E038] the provider `<Provider>` is not declared
```

### CGP-E039

A plain `impl` implements a provider trait. The trait has `IsProviderFor` as a supertrait, and only `#[cgp_provider]` or
`#[cgp_impl]` writes the provider's impl of it. To fix it, add one of those macros, as the `help` line says.
[`#[cgp_provider]`](/docs/reference/macros/cgp_provider) describes the macro.

```text
[CGP-E039] the impl of `<Trait>` for `<Provider>` is written without `#[cgp_provider]`
```

## Wiring keys and values that do not resolve

These codes replace a name-resolution error on a wiring entry, for a name the tool can settle. Each applies only when no
type of that name exists in your crate or its dependencies, or when the tool can say which name you meant.
[`delegate_components!`](/docs/reference/macros/delegate_components) describes what a key and a value may be.

### CGP-E040

A wiring key names a consumer trait such as `CanGreet`, where its component marker `GreeterComponent` belongs. To fix it, name
the component, as the `help` line says.

```text
[CGP-E040] the wiring key `<Consumer>` is a consumer trait, but a key names a component
```

### CGP-E041

A key names a component that exists but is not imported where the table is written, such as `ErrorTypeProviderComponent`,
which the prelude does not export. To fix it, add the `use` line the `help` line gives.

```text
[CGP-E041] the component `<Component>` is not in scope
```

### CGP-E042

A key was guessed from a consumer trait's name with `Component` appended, such as `HasScalarTypeComponent`, while the marker
follows the provider trait, as in `ScalarTypeProviderComponent`. To fix it, name the component the `help` line gives.

```text
[CGP-E042] the wiring key `<Key>` names no component
```

### CGP-E043

A wiring value names a provider struct nothing declares, often a misspelling. To fix it, declare the provider or name one that
exists. A `help` line appears when the component has a provider with a similar name. A provider declared elsewhere but not
imported keeps the compiler's `E0425` and its import suggestion instead.

```text
[CGP-E043] the provider `<Provider>` wired for `<Key>` is not declared
```

### CGP-E044

An `@`-path in a key, or in the value of a `=>` redirect, names a type nothing declares. A path segment that is not a lowercase
identifier stays the type it spells, and only a lowercase one becomes a symbol. So to fix it, declare or import the type, or
write the segment in lowercase. A type declared elsewhere but not imported keeps the compiler's `E0425` and its import
suggestion instead.

```text
[CGP-E044] the type `<Name>` in a wiring path is not declared
```

## Extensible data

### CGP-E050

A cast, builder, or extractor operation is called on a value whose field or variant is not in the state the operation needs, or
on a type without the data derive it needs. The headline states the cause, and the chain ends at one of the codes from
`[CGP-E118]` to `[CGP-E126]`, at `[CGP-E110]` for a missing field, or at `[CGP-E117]` for an ordinary bound. For example:

```text
[CGP-E050] missing variant `Bar` on `Big`
```

Its class sentence is `` the trait `<Trait>` is not implemented for `<Type>` ``. To fix it, follow the cause: drop a second
setting or extraction, set an unset field, add a missing variant or align a payload type, or add the derive or call the method a
`help` line names. [Extensible variants](/docs/concepts/extensible-variants) and
[extensible records](/docs/concepts/extensible-records) explain what those operations require.

## Steps of the dependency chain

The rest of the codes label the lines of the tree beneath a `root cause:` note. These four are steps along the way, and they need
no fix of their own.

### CGP-E101

This line is a step through a context's consumer trait, and it is usually the first line of the chain. A type without wiring of
its own is named plainly, without the word "context".

```text
[CGP-E101] consumer trait impl `<Consumer>` for context `<Context>`
```

### CGP-E102

This line is a step through a provider's provider trait: the context's wiring selected this provider, and the chain continues with
what the provider needs. Directly above a `[CGP-E115]` line for the same provider, which has no such impl, it reads
`` provider trait `<ProviderTrait>` required of provider `<Provider>` with context `<Context>` ``.

```text
[CGP-E102] provider trait impl `<ProviderTrait>` with context `<Context>` for provider `<Provider>`
```

### CGP-E103

This line is a step through a redirected lookup, from an `open` statement or a namespace, naming the table the path is looked up in.

```text
[CGP-E103] redirect lookup to `@<path>` in `<Table>`
```

### CGP-E104

This line is a step through any other trait, such as a getter or a blanket trait you wrote.

```text
[CGP-E104] trait impl `<Trait>` for `<Type>`
```

## Where the chain ends

These codes end the chain, and the cause they state is the one to fix. The `root cause:` line repeats the same code.

### CGP-E110

The chain ends at a field the context does not have. To fix it, add the field, or wire a provider that does not read it. A `help`
line names a similar field when there is one, and otherwise the field to add.
[A dependency is not met](/docs/reference/errors#a-dependency-is-not-met) explains the class.

```text
[CGP-E110] missing field `<field>` on `<Type>`
```

### CGP-E111

The chain ends at a field that exists but has no `HasField` impl, because the struct does not derive it. To fix it, add
`#[derive(HasField)]` to the struct, as the `help` line says.
[When the derive is missing entirely](/docs/reference/errors#when-the-derive-is-missing-entirely) explains the class.

```text
[CGP-E111] `HasField` not derived for field `<field>` of `<Type>`
```

### CGP-E112

The chain ends at a field with the wrong type. This is the last line of a `[CGP-E004]` error, with the same wording.

```text
[CGP-E112] field `<field>` on `<Type>` has type `<actual>`, but `<expected>` is required
```

### CGP-E113

The chain ends at a component or path the context wires nothing for. To fix it, add the wiring entry the `help` line names.
[Nothing is wired there](/docs/reference/errors#nothing-is-wired-there) explains the class.

```text
[CGP-E113] missing wiring for `<key>` on `<Context>`
```

### CGP-E114

The chain ends at a provider's own table that has no entry for a key: a bundle of wiring missing a component, or a provider that
dispatches per type missing the type it was given. To fix it, add the entry to that provider's table, or pass it a type the table
covers.

```text
[CGP-E114] missing entry for `<key>` in provider `<Provider>`
```

### CGP-E115

The chain ends at a type wired where a provider belongs that is not a provider, such as a value type put in a provider's place. To
fix it, wire an actual provider in that position. A `help` line suggests one when the program supports it: an adapter such as
`WithProvider`, `UseFields` in place of `UseField` on a multi-method getter, an `open` statement in place of `UseDelegate`, or the
right component for a provider of another one.

```text
[CGP-E115] `<Type>` is not a provider of `<ProviderTrait>`
```

### CGP-E116

The chain ends at an associated type that differs from what was required. This is the last line of a `[CGP-E005]` error, with the
same wording.

```text
[CGP-E116] abstract type `<Type>` of `<Trait>` on `<Owner>` is `<actual>`, but `<expected>` is required
```

### CGP-E117

The chain ends at an ordinary Rust trait bound that does not hold, such as `f64: Eq`, in the compiler's own wording. At a
`check_components!` entry the tool keeps the compiler's headline, which already names the bound. To fix it, use a type that implements
the trait, or a provider that does not require it.
[When the missing bound is an ordinary Rust trait](/docs/reference/errors#when-the-missing-bound-is-an-ordinary-rust-trait) explains the
class.

```text
[CGP-E117] the trait bound `<Type: Trait>` is not satisfied
```

### CGP-E118

A builder operation needs a field unset, as a second `build_field` or a second merge source does, and the field is already set. To fix
it, drop the second setting.

```text
[CGP-E118] field `<field>` of `<Type>` is already set
```

### CGP-E119

An operation such as a finalize or `take_field` needs one or more fields set, and they are not. To fix it, set them first.

```text
[CGP-E119] field `<field>` of `<Type>` is not set
```

### CGP-E120

A cast between enums has variants that do not match, or a construction uses a tag the enum lacks. To fix it, add the variant or correct
the name. A `help` line names a similar variant when there is one.

```text
[CGP-E120] missing variant `<Variant>` on `<Type>`
```

### CGP-E121

An extractor is finalized while it may still yield a variant, as a matcher that leaves one out does. To fix it, handle the variant.

```text
[CGP-E121] variant `<Variant>` of `<Type>` is not extracted
```

### CGP-E122

A cast between enums has same-named variants that carry different payloads. To fix it, align the payload types.

```text
[CGP-E122] variant `<Variant>` on `<Type>` has type `<actual>`, but `<expected>` is required
```

### CGP-E123

A trait only a data derive implements is required of a type that does not have it. To fix it, add the derive the `help` line names.

```text
[CGP-E123] `<Type>` does not implement `<Trait>`
```

### CGP-E124

A variant is extracted a second time from an extractor that no longer holds it. The compiler reports this as an `E0308` on the tag
argument. To fix it, drop the second extraction.

```text
[CGP-E124] variant `<Variant>` of `<Type>` is already extracted
```

### CGP-E125

An optional builder's operation is called on a field a core builder holds set or unset. To fix it, call `to_optional` on the builder
first, as the `help` line says.

```text
[CGP-E125] field `<field>` of `<Type>` is not optional
```

### CGP-E126

A builder or extractor is given to an operation that needs a trait only its struct or enum implements, as `downcast` on the remainder
of a downcast is. To fix it, call the method the `help` line names, such as `downcast_fields`.

```text
[CGP-E126] an extractor of `<Type>` does not implement `<Trait>`
```

### CGP-E127

A provider's impls differ from the wiring in exactly one argument: the context, or a component parameter, as for a pipeline stage that
takes `u64` after one that returns `String`. To fix it, wire a provider for those arguments, or change the type that disagrees.

```text
[CGP-E127] `<Provider>` implements `<ProviderTrait>` only for `<A>`, not `<B>`
```

### CGP-E128

A hand-written `DelegateComponent` impl lacks the `IsProviderFor` impl `delegate_components!` writes beside each entry, so a check passes
but every consumer call fails. To fix it, replace the impl with `delegate_components!`, as the `help` line says.

```text
[CGP-E128] the wiring of `<Component>` on `<Context>` has no `IsProviderFor` impl
```

### CGP-E129

A consumer method is called on a value of a type parameter without the bound. To fix it, add the bound the `help` line names.

```text
[CGP-E129] the type parameter `<T>` of `<function>` is not bounded by `<Trait>`
```

## Rewrites without a code

Some changes make an error easier to read without classifying it, so they carry no code. You may see them in an error whose headline the
tool left as the compiler wrote it:

- **Type-level names written back as you wrote them:** a field name reads `Symbol!("name")`, a list reads `Product![A, B]` or `Sum![A, B]`,
  and a path reads `Path!(@app.Component)`. A list of named or numbered fields reads as the struct or enum body it describes, such as
  [`Struct! { name: String }`](/docs/reference/macros/struct) or [`Enum! { Circle(f64) }`](/docs/reference/macros/enum), when it has such a
  spelling.
- **Shorter type names:** the module paths the compiler prints in front of a type go, except a qualifier that tells two types apart, such
  as `std::io::Error` beside a crate's own `Error`.
- **A missing-field sentence:** an unmet `HasField` requirement inside a note reads as a missing field.
- **Consumer and provider trait names:** the compiler's "required for … to implement …" notes name them in place of the generated traits
  that connect them.
- **Removed advice:** the compiler's suggestion that a method call should use associated-function syntax goes, because it does not fix the
  mistake for CGP's provider methods.
- **A better rename:** on a misspelled `#[use_type]` import, the compiler's suggestion to respell the name becomes one to rename it in the
  import and in each use.

---

*An AI agent wrote this page using the CGP knowledge base. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
