---
title: 'cargo-cgp error codes: every CGP-E code explained'
sidebar_label: 'Error codes'
sidebar_position: 8
description: 'Look up any [CGP-Exxx] code cargo cgp check prints: what the message says, what mistake it means, how to fix it, and where the class is explained.'
---

# Error codes

A code such as `[CGP-E001]` identifies the kind of failure `cargo cgp check` recognized. This page
lists those codes with their message forms, causes, and remedies. The command belongs to the
[`cargo-cgp`](./index.md) toolchain for [Context-Generic Programming (CGP)](/docs/).
[Reading the output](./reading-output.md) places the codes in complete diagnostics, while the
[compile-errors reference](/docs/reference/errors) explains the underlying mistake classes.

## How the codes work

The CGP code supplements the compiler's own code. In `error[E0277]: [CGP-E001] …`, `E0277` still
refers to the explanation available through `rustc --explain`, while `[CGP-E001]` identifies the
CGP failure. Searching for a CGP code therefore collects the rewritten errors of that class.

The hundreds digit distinguishes whole-error codes from dependency steps. A `CGP-E0xx` code
heads the diagnostic, while a `CGP-E1xx` code labels a line of its dependency tree. Codes
`CGP-E101` through `CGP-E104` describe the path through requirements; codes from `CGP-E110` onward
identify the cause at the end.

In the messages below, a word in angle brackets, such as `<Context>`, stands for the name the tool fills in
from your code.

## A trait the context cannot use

The first family covers a requirement that fails somewhere under a trait. The tool traces the failure to its
causes, so the error usually carries a `root cause:` note and a dependency chain.

### CGP-E001

The context cannot obtain the consumer trait, either because the component has no wiring entry
or because the chosen provider lacks a dependency. The headline names the cause using the same
wording as the corresponding dependency-chain line:

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

The selected provider cannot implement its provider trait for this context because one of its
requirements fails. This is the provider-side form of `[CGP-E001]`, reported by a failing
`#[check_providers(...)]` assertion in [`check_components!`](/docs/reference/macros/check_components)
or a call through the provider's own path. Its headline names the cause, using this class sentence
when causes differ:

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

The same key is wired twice, because a wiring table or a namespace maps one component or path to two providers.

```text
[CGP-E010] conflicting wiring: `<Context>` wires <key> twice
```

To fix it, remove one of the entries.

### CGP-E011

Two different keys overlap, so an entry is wired that another already covers. It may be a generic entry over a
specific one, a path that covers a longer path, or an entry for a key the context's namespace already binds.

```text
[CGP-E011] conflicting wiring: `<Context>` wires <key>, which <source> already covers
```

To fix it, remove or narrow the overlapping entry.
[Overriding something a namespace already binds](/docs/reference/errors#overriding-something-a-namespace-already-binds)
covers the namespace case.

The source is the key that covers the other, whichever comes first. When neither covers the other, the key is the
one they share.

### CGP-E012

A context joins more than one namespace. Each namespace covers every key, so two of them, or a namespace and a loop
over bare keys, conflict.

```text
[CGP-E012] conflicting wiring: `<Context>` joins both `<A>` and `<B>`, but a target type in `delegate_components!` can join only one namespace
```

To fix it, join one namespace and make it inherit the others.
[Joining more than one namespace](/docs/reference/errors#joining-more-than-one-namespace) explains the class.

### CGP-E013

A direct entry collides with a redirect of the same key, such as a component wired by name while an `open`
statement, a `=>` redirect, or a namespace sends it along a path.

```text
[CGP-E013] conflicting wiring: `<Context>` wires <component> directly, but it is redirected to `<path>`
```

To fix it, wire the provider under the redirected
path rather than the component name, as the `help` line says.
[Overriding something a namespace already binds](/docs/reference/errors#overriding-something-a-namespace-already-binds)
explains the class.

### CGP-E014

The same key is redirected twice, by two `open` statements or `=>` mappings on one context, or by one path registered
twice in a namespace.

```text
[CGP-E014] conflicting wiring: `<Context>` redirects <component> twice, to `<path>` and to `<path>`
```

To fix it, keep one redirect.

## Structural mistakes

These codes name a mistake in the shape of what a macro was given, which the compiler rejects through generated items.
Each one uses your program's terms, drops the compiler's suggestions that do not apply, and gives the fix in a `help`
line.

### CGP-E015

A `check_components!` table lists one entry twice.

```text
[CGP-E015] `check_components!` checks `<entry>` twice for `<Context>`
```

To fix it, remove one of the two. A `delegate_and_check_components!` table that lists an entry twice also wires it twice,
so you see that as [CGP-E010](#cgp-e010) alone.
[The wiring itself does not compile](/docs/reference/errors#the-wiring-itself-does-not-compile) covers this family.

### CGP-E016

Checking the same context in two `check_components!` tables, or two
`delegate_and_check_components!` tables, in one module gives both checks the same generated
trait name. The message identifies the macro used:

```text
[CGP-E016] two `check_components!` tables in one module check `<Context>`
```

Merge the tables or give one check a distinct name with `#[check_trait(...)]`. If two attributes
explicitly choose the same name, the message identifies that conflict instead. For tables from
different macros, it names both and asks you to rename one check trait.
[The wiring itself does not compile](/docs/reference/errors#the-wiring-itself-does-not-compile)
covers these cases.

### CGP-E017

A `for` loop in the wiring binds a variable its key does not use.

```text
[CGP-E017] the `for` loop in the wiring of `<Context>` binds `<T>`, but its key `<key>` does not use it
```

To fix it, write the variable in the key where the
type the loop covers goes.
[A generic that reaches nothing](/docs/reference/errors#a-generic-that-reaches-nothing) explains the class.

### CGP-E018

A `#[default_impl]` registers a provider whose impl is generic over a parameter the key does not fix.

```text
[CGP-E018] `#[default_impl]` cannot register `<Provider>` for `<key>`, because the provider's impl is generic over `<T>`
```

To fix it, write
the impl for that key alone, as the `help` line says.
[A generic that reaches nothing](/docs/reference/errors#a-generic-that-reaches-nothing) explains the class.

### CGP-E019

A wiring or namespace entry binds a generic parameter its key does not use.

```text
[CGP-E019] the entry for `<key>` in the wiring of `<Context>` binds `<T>`, but its key does not use it
```

To fix it, write a concrete type in place of
the parameter and remove it from the entry's generic parameters, as the `help` line says.
[A generic that reaches nothing](/docs/reference/errors#a-generic-that-reaches-nothing) explains the class.

### CGP-E060

Omitting parameters from a check makes it request the component at unit parameters `()`.
If the provider supports a different parameter set, that check cannot establish the intended
implementation:

```text
[CGP-E060] the check of `<entry>` on `<Context>` names no parameters, so it checks the component at the unit parameters `()`
```

To fix it, check it at the parameters the `help` line offers, with
`#[check_params(...)]` in `delegate_and_check_components!` or `<entry>: <P>` in `check_components!`.
[`check_components!`](/docs/reference/macros/check_components) describes the syntax.

### CGP-E061

The generated names for [`#[cgp_auto_dispatch]`](/docs/reference/macros/cgp_auto_dispatch) depend
on the method name. Two traits declaring the same method in one module therefore produce
colliding generated items:

```text
[CGP-E061] two `#[cgp_auto_dispatch]` traits in one module declare the method `<method>`
```

To fix it, rename the method in one trait, or
declare the traits in different modules.

## Wiring that never resolves, and the orphan rule

The next three codes cover wiring that is well formed but cannot work, either because looking it up never ends or because
Rust's coherence rules forbid it.

### CGP-E020

Looking up the wiring for a component leads back to itself. The plain case is a component wired to `UseContext` on a
context with no other implementation of it. The compiler's code is `E0275`.

```text
[CGP-E020] the wiring for the consumer trait `<Consumer>` on context `<Context>` never resolves, because the lookup leads back to itself
```

To fix it, wire a real provider, or implement
the consumer trait directly on the context. The `help` line names the provider that closes the loop.
[A lookup that never terminates](/docs/reference/errors#a-lookup-that-never-terminates) explains the class.

### CGP-E021

The lookup keeps requesting the consumer trait for a larger type. Because each request differs,
Rust eventually reaches its recursion limit without finding an implementation. Raising that
limit cannot terminate an ever-growing lookup; select a provider whose requirements stop growing.
[A lookup that never terminates](/docs/reference/errors#a-lookup-that-never-terminates)
explains this failure.

```text
[CGP-E021] the wiring for the consumer trait `<Consumer>` on context `<Context>` never resolves, because each step requires it for a larger type (`<Larger>`, …)
```

### CGP-E022

The registration belongs to neither end of the lookup: both its namespace and its key come from
other crates. Rust's orphan rule rejects this with `E0210` or `E0117`:

```text
[CGP-E022] cannot register the foreign <key> into the foreign namespace `<Namespace>`
```

Make one end local, as the `help` line suggests. You can use a component defined in your crate,
move the registration into the namespace's crate, or define a local namespace inheriting the
foreign one.
[Registering into a namespace you do not own](/docs/reference/errors#registering-into-a-namespace-you-do-not-own)
explains the ownership requirement.

## Mistakes in what a macro was given

These codes name a CGP attribute or macro used in a way the generated code cannot compile. Most of them give the fix in a
`help` line.

### CGP-E030

A `#[cgp_fn]` or `#[cgp_impl]` body can call a CGP method on `self` or `Self` only when its context
bounds include the trait. The macro generates a generic context, so bringing the trait into scope
alone does not establish that requirement. Omitting the dependency produces `E0599`:

```text
[CGP-E030] the trait `<Trait>` is used but not declared as a dependency
```

To fix it, add the trait to `#[uses(...)]`, as the
`help` line says.
[Using something you did not declare](/docs/reference/errors#using-something-you-did-not-declare) explains the class.

### CGP-E031

The higher-order provider calls an inner provider without importing its provider-trait bound.
Without that requirement, Rust cannot resolve the inner call:

```text
[CGP-E031] the inner provider `<Inner>` is used but not imported
```

To fix it, import it with `#[use_provider(<Inner>: <ProviderTrait>)]`, as the `help` line says.
[Using something you did not declare](/docs/reference/errors#using-something-you-did-not-declare) explains the class.

### CGP-E032

A `#[cgp_impl]` header names the component's consumer trait where its provider trait belongs. Without the rewrite, this one
mistake produces a burst of unrelated-looking errors.

```text
[CGP-E032] `<Consumer>` is a consumer trait, but a `#[cgp_impl]` provider must implement its provider trait `<ProviderTrait>`
```

To fix it, name the provider trait in the header, as the `help` line
says. [Naming the wrong half of a component](/docs/reference/errors#naming-the-wrong-half-of-a-component) explains the
class.

### CGP-E033

The trait named by `#[cgp_impl]` is not a CGP component, so it has no provider trait for the macro
to implement:

```text
[CGP-E033] `#[cgp_impl]` can only implement a CGP component's provider trait, but `<Trait>` is not a CGP component
```

To fix it, make the trait a component with [`#[cgp_component]`](/docs/reference/macros/cgp_component) and
implement its provider trait, or drop `#[cgp_impl]` and write an ordinary `impl`.
[Naming the wrong half of a component](/docs/reference/errors#naming-the-wrong-half-of-a-component) explains
the class.

### CGP-E034

An inner-provider bound names a consumer trait where the provider trait belongs, most often in
[`#[use_provider]`](/docs/reference/attributes/use_provider).

```text
[CGP-E034] `<Consumer>` is a consumer trait and cannot bound an inner provider; a higher-order provider imports its provider trait `<ProviderTrait>`
```

To fix it, name the provider trait, as the `help` line says.
[Naming the wrong half of a component](/docs/reference/errors#naming-the-wrong-half-of-a-component) explains the class.

### CGP-E035

A `#[cgp_fn]`, `#[cgp_impl]`, or `#[cgp_component]` definition names an abstract type of the context, as `Self::Error` or a
bare `Error`, without [`#[use_type]`](/docs/reference/attributes/use_type).

```text
[CGP-E035] the abstract type `<Type>` is used but not imported
```

To fix it, import the type with
`#[use_type(<Trait>.<Type>)]`, as the `help` line says, and write it bare.
[Using something you did not declare](/docs/reference/errors#using-something-you-did-not-declare) explains the class.

### CGP-E036

A component can require a trait of every context through
[`#[extend(...)]`](/docs/reference/attributes/extend) or `#[use_type(...)]`. Its provider must
declare that requirement too, even if the provider's method bodies never use it:

```text
[CGP-E036] the provider trait `<ProviderTrait>` requires `<Trait>` of the context, but the provider `<Provider>` does not declare it
```

To fix it, add
the `#[uses(...)]` or `#[use_type(...)]` the `help` line names.
[Using something you did not declare](/docs/reference/errors#using-something-you-did-not-declare) explains the class.

### CGP-E037

[`#[uses]`](/docs/reference/attributes/uses) names a provider trait, but a dependency names its consumer trait.

```text
[CGP-E037] `<ProviderTrait>` is a provider trait and cannot be a `#[uses]` dependency; a dependency names its consumer trait `<Consumer>`
```

To fix it,
name the consumer trait, as the `help` line says.
[Naming the wrong half of a component](/docs/reference/errors#naming-the-wrong-half-of-a-component) explains the class.

### CGP-E038

The form `#[cgp_impl(GreetHello)]` implements an existing provider struct. If nothing declares
`GreetHello`, that implementation has no type to target:

```text
[CGP-E038] the provider `<Provider>` is not declared
```

Use `#[cgp_impl(new GreetHello)]` to declare the struct with the implementation, or declare it
separately. If a provider is declared elsewhere but not imported, the tool instead keeps the
compiler's `E0425` and import suggestion.
[`#[cgp_impl]`](/docs/reference/macros/cgp_impl) describes both declaration forms.

### CGP-E039

The provider trait requires `IsProviderFor` as a supertrait, but a plain `impl` does not supply
that implementation. `#[cgp_provider]` and `#[cgp_impl]` generate it alongside the provider impl:

```text
[CGP-E039] the impl of `<Trait>` for `<Provider>` is written without `#[cgp_provider]`
```

To fix it, add one of those macros, as the `help` line says.
[`#[cgp_provider]`](/docs/reference/macros/cgp_provider) describes the macro.

## Wiring keys and values that do not resolve

A wiring name may fail to resolve because it is missing, not imported, or the wrong kind of
item. These codes replace the compiler's message only when the tool can establish that no such
type exists in the crate or its dependencies, or can identify the intended name.
[`delegate_components!`](/docs/reference/macros/delegate_components) explains the permitted keys
and values.

### CGP-E040

The table uses a consumer trait such as `CanGreet` as its key. Wiring needs the corresponding
component marker, such as `GreeterComponent`, to select a provider:

```text
[CGP-E040] the wiring key `<Consumer>` is a consumer trait, but a key names a component
```

Replace the trait with the component marker named by the `help` line.

### CGP-E041

The component exists, but the table cannot see it. For example, `ErrorTypeProviderComponent`
needs an explicit import because it is not exported by the prelude:

```text
[CGP-E041] the component `<Component>` is not in scope
```

To fix it, add the `use` line the `help` line gives.

### CGP-E042

The key uses a consumer trait's name with `Component` appended, such as `HasScalarTypeComponent`.
The actual marker follows the provider trait's name instead, as in `ScalarTypeProviderComponent`:

```text
[CGP-E042] the wiring key `<Key>` names no component
```

To fix it, name the component the `help` line gives.

### CGP-E043

The table selects a provider struct that has not been declared, often because its name is
misspelled:

```text
[CGP-E043] the provider `<Provider>` wired for `<Key>` is not declared
```

Declare the intended provider or select an existing one. When the component has a provider with
a similar name, a `help` line suggests it. A provider that exists elsewhere but is not imported
keeps the compiler's `E0425` and import suggestion instead.

### CGP-E044

A segment in an `@` key or `=>` destination names an undeclared type. Lowercase identifiers
become symbols, while other segments retain their spelling as types, so capitalization can
change the lookup's meaning. Declare or import the intended type, or use lowercase for a symbol.
An existing type that is merely unimported keeps the compiler's `E0425` and import suggestion.

```text
[CGP-E044] the type `<Name>` in a wiring path is not declared
```

## Extensible data

### CGP-E050

The requested cast, build, or extraction cannot operate on the value in its current state, or
the type lacks the needed data derive. The headline identifies that cause. The chain ends at
`[CGP-E118]` through `[CGP-E126]` for data-state failures, `[CGP-E110]` for a missing field, or
`[CGP-E117]` for an ordinary bound. For example:

```text
[CGP-E050] missing variant `Bar` on `Big`
```

The general class sentence is `` the trait `<Trait>` is not implemented for `<Type>` ``.
Follow the reported cause to choose the correction: remove a duplicate setting or extraction,
set a missing field, supply a compatible variant or payload, or use the derive or method named
in `help`. [Extensible variants](/docs/concepts/extensible-variants) and
[extensible records](/docs/concepts/extensible-records) explain the state each operation requires.

## Steps of the dependency chain

The rest of the codes label the lines of the tree beneath a `root cause:` note. These four are steps along the way, and they need
no fix of their own.

### CGP-E101

The chain usually begins with the consumer trait required of the context. If the type has no
wiring of its own, the message names it without the word “context”.

```text
[CGP-E101] consumer trait impl `<Consumer>` for context `<Context>`
```

### CGP-E102

The wiring selected this provider, so the chain proceeds through its provider-trait requirements.
If the provider lacks the implementation entirely, a `[CGP-E115]` line follows and this step
instead reads `` provider trait `<ProviderTrait>` required of provider `<Provider>` with context `<Context>` ``.

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
The codes from `[CGP-E118]` to `[CGP-E126]` belong to extensible data, which
[extensible records](/docs/concepts/extensible-records) and [extensible variants](/docs/concepts/extensible-variants)
explain.

### CGP-E110

The chain ends at a field the context does not have.

```text
[CGP-E110] missing field `<field>` on `<Type>`
```

Add the required field or select a provider that does not read it. The `help` line points to a
similar existing field when possible; otherwise, it names the field to add.
[A dependency is not met](/docs/reference/errors#a-dependency-is-not-met) explains this failure.

### CGP-E111

The field is present, but generic field access is unavailable because the struct has not derived
`HasField`:

```text
[CGP-E111] `HasField` not derived for field `<field>` of `<Type>`
```

Add `#[derive(HasField)]` to expose the struct's fields, as the `help` line suggests.
[When the derive is missing entirely](/docs/reference/errors#when-the-derive-is-missing-entirely)
explains the missing access implementation.

### CGP-E112

The chain ends at a field with the wrong type. This is the last line of a `[CGP-E004]` error, with the same wording.

```text
[CGP-E112] field `<field>` on `<Type>` has type `<actual>`, but `<expected>` is required
```

### CGP-E113

The chain ends at a component or path the context wires nothing for.

```text
[CGP-E113] missing wiring for `<key>` on `<Context>`
```

To fix it, add the wiring entry the `help` line names.
[Nothing is wired there](/docs/reference/errors#nothing-is-wired-there) explains the class.

### CGP-E114

The context selected a provider, but that provider's table lacks the requested key. For a bundle,
this can mean a missing component; for per-type dispatch, a missing entry for the supplied type:

```text
[CGP-E114] missing entry for `<key>` in provider `<Provider>`
```

To fix it, add the entry to that provider's table, or pass it a type the table
covers.

### CGP-E115

The selected type cannot supply the required provider trait. A common cause is putting a value
type in the table where a provider belongs:

```text
[CGP-E115] `<Type>` is not a provider of `<ProviderTrait>`
```

Select a provider for that component. When the program supports a specific correction, `help`
may suggest an adapter such as `WithProvider`, `UseFields` for a multi-method getter instead
of `UseField`, an `open` statement instead of `UseDelegate`, or the component that matches
the provider you selected.

### CGP-E116

The chain ends at an associated type that differs from what was required. This is the last line of a `[CGP-E005]` error, with the
same wording.

```text
[CGP-E116] abstract type `<Type>` of `<Trait>` on `<Owner>` is `<actual>`, but `<expected>` is required
```

### CGP-E117

An ordinary Rust bound, such as `f64: Eq`, failed at the end of the dependency chain. The line
keeps the compiler's wording. At a `check_components!` entry, the original headline already
identifies this bound, so the tool keeps it too.

```text
[CGP-E117] the trait bound `<Type: Trait>` is not satisfied
```

To fix it, use a type that implements
the trait, or a provider that does not require it.
[When the missing bound is an ordinary Rust trait](/docs/reference/errors#when-the-missing-bound-is-an-ordinary-rust-trait) explains the
class.

### CGP-E118

A field can be supplied only while it is unset. A second `build_field` call or a merge source
supplying the same field therefore fails once the builder already contains its value:

```text
[CGP-E118] field `<field>` of `<Type>` is already set
```

To fix it, drop the second setting.

### CGP-E119

Finalization and operations such as `take_field` require the relevant fields to be present.
One or more are still unset in this builder:

```text
[CGP-E119] field `<field>` of `<Type>` is not set
```

To fix it, set them first.

### CGP-E120

An enum conversion cannot find a compatible variant, or construction uses a tag absent from
the target enum:

```text
[CGP-E120] missing variant `<Variant>` on `<Type>`
```

To fix it, add the variant or correct
the name. A `help` line names a similar variant when there is one.

### CGP-E121

Finalization requires an extractor with no possible variants remaining. An unhandled case,
such as a missing matcher branch, leaves this variant available:

```text
[CGP-E121] variant `<Variant>` of `<Type>` is not extracted
```

To fix it, handle the variant.

### CGP-E122

The enums share this variant's name, but disagree on its payload type. That prevents the cast
from transferring the payload:

```text
[CGP-E122] variant `<Variant>` on `<Type>` has type `<actual>`, but `<expected>` is required
```

To fix it, align the payload types.

### CGP-E123

A trait only a data derive implements is required of a type that does not have it.

```text
[CGP-E123] `<Type>` does not implement `<Trait>`
```

To fix it, add the derive the `help` line names.

### CGP-E124

The extractor has already excluded this variant, so extracting it again is invalid. Rust reports
the mismatch as `E0308` on the tag argument:

```text
[CGP-E124] variant `<Variant>` of `<Type>` is already extracted
```

To fix it, drop the second extraction.

### CGP-E125

An optional builder's operation is called on a field a core builder holds set or unset.

```text
[CGP-E125] field `<field>` of `<Type>` is not optional
```

To fix it, call `to_optional` on the builder
first, as the `help` line says.

### CGP-E126

The method requires a trait implemented by the original struct or enum, but receives a builder
or extractor instead. Calling `downcast` on a previous downcast's remainder is one example:

```text
[CGP-E126] an extractor of `<Type>` does not implement `<Trait>`
```

To fix it, call the method the `help` line names, such as `downcast_fields`.

### CGP-E127

The wiring requests arguments that differ from the provider's implementations in exactly one
position: either the context or a component parameter. For example, a pipeline stage may accept
`u64` while the previous stage returns `String`:

```text
[CGP-E127] `<Provider>` implements `<ProviderTrait>` only for `<A>`, not `<B>`
```

To fix it, wire a provider for those arguments, or change the type that disagrees.

### CGP-E128

Writing `DelegateComponent` by hand can leave out the accompanying `IsProviderFor` implementation
that `delegate_components!` generates. In that case, a component check can pass while every
consumer call fails:

```text
[CGP-E128] the wiring of `<Component>` on `<Context>` has no `IsProviderFor` impl
```

To fix it, replace the impl with `delegate_components!`, as the `help` line says.

### CGP-E129

A consumer method is called on a value of a type parameter without the bound.

```text
[CGP-E129] the type parameter `<T>` of `<function>` is not bounded by `<Trait>`
```

To fix it, add the bound the `help` line names.

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
