---
title: 'Check wiring where the choice is written'
sidebar_label: 'Checking your wiring'
sidebar_position: 6
description: 'How check traits and check_components! verify a context wiring beside the table, what the raw and rewritten diagnostics show, and what a check does not prove.'
---

# Checking your wiring

[Context-Generic Programming](/docs/) (CGP) extends Rust with pluggable trait implementations, and
a **check trait** verifies that a context can use the providers its wiring selects. Wiring itself
records provider choices without immediately checking whether the context meets their requirements,
so a missing dependency can remain unnoticed until a later method call. An explicit check brings
that failure back beside the wiring, where the context and its selected providers are easier to
inspect.

## A context can be wrong and still compile

Consider a provider that records outgoing email in a `sent_emails` field. Its wiring can still
name a context with only an `smtp_server` field, as this example shows. The fragment imports
CGP's prelude and defines both the `EmailSender` component and the recording provider:

```rust
use cgp::prelude::*;
use core::cell::RefCell;

#[cgp_component(EmailSender)]
pub trait CanSendEmail {
    fn send_email(&self, to: &str, body: &str);
}

#[cgp_impl(new RecordEmails)]
impl EmailSender {
    fn send_email(&self, #[implicit] sent_emails: &RefCell<Vec<String>>, to: &str, body: &str) {
        sent_emails.borrow_mut().push(format!("{to}: {body}"));
    }
}

#[derive(HasField)]
pub struct BrokenApp {
    pub smtp_server: String,
}

delegate_components! { BrokenApp { EmailSenderComponent: RecordEmails } }
```

This table establishes which provider `BrokenApp` selects without proving that it can use that
provider. The missing proof would require the `sent_emails` field read by `RecordEmails`' implicit
argument. Because this fragment never asks for `CanSendEmail` on `BrokenApp`, the missing field has
not yet caused a compilation failure.

Deferring that proof is useful for reusable wiring. An [aggregate provider](./aggregate-providers.md)
can collect choices before the contexts that use it are even defined. Those contexts must eventually
meet the providers' requirements, but the bundle itself needs only to record the choices.

## Where the failure surfaces instead

An unchecked wiring error appears when code first requires the trait. A call to
`app.send_email(to, body)` on `&BrokenApp` produces an error like this abbreviated diagnostic:

```text
error[E0599]: the method `send_email` exists for reference `&BrokenApp`,
              but its trait bounds were not satisfied
   |
   | pub struct BrokenApp { pub smtp_server: String }
   | -------------------- doesn't satisfy `BrokenApp: CanSendEmail`
   |                      or `BrokenApp: EmailSender<BrokenApp>`
   |
note: the following trait bounds were not satisfied:
      `BrokenApp: EmailSender<BrokenApp>`
```

The diagnostic stops at the unsatisfied trait implementations: `BrokenApp` lacks `CanSendEmail`
and `EmailSender<BrokenApp>`. It leaves the missing `sent_emails` field hidden behind the generated
implementations. If the first call occurs in another module or crate, the error can also appear a
long way from the table that selected the provider.

## Asking the question at a line you chose

A check trait forces the compiler to verify a bound where you write its implementation. The basic
technique uses an ordinary Rust [supertrait](/docs/reference/glossary#supertrait) and an empty implementation:

```rust
trait CanUseApp: CanSendEmail {}
impl CanUseApp for App {}
```

For Rust to accept the empty implementation, it must establish `App: CanSendEmail` at that line.
Putting it beside the wiring moves a failure there. But the requirement still goes through the
consumer trait, so its diagnostic can remain as vague as the method-call error.

To expose the provider's requirements, the check needs a path through more than the consumer
trait. [`check_components!`](/docs/reference/macros/check_components) generates that check:

```rust
check_components! {
    BrokenApp {
        EmailSenderComponent,
    }
}
```

The generated assertion asks for `CanUseComponent`. Satisfying this bound requires both a wiring
entry and the chosen provider's dependencies for the context being checked. The missing field
therefore appears in the diagnostic:

```text
error[E0277]: the trait bound `BrokenApp: CanUseComponent<EmailSenderComponent>`
              is not satisfied
   |
help: the trait `HasField<Symbol<_, Chars<_, Chars<'e', Chars<'n', Chars<'t', …>>>>>>`
      is not implemented for `BrokenApp`
      but trait `HasField<Symbol<_, Chars<_, Chars<'m', Chars<'t', Chars<'p', …>>>>>>`
      is implemented for it
   |
note: required for `RecordEmails` to implement
      `IsProviderFor<EmailSenderComponent, BrokenApp>`
```

Following this checking path exposes `RecordEmails`' requirement for a `HasField` implementation.
The cause is closer to the surface, but the field tags still appear as nested character types,
leaving `sent_emails` and `smtp_server` difficult to recognize. The headline also refers to the
generated `CanUseComponent` bound rather than the public trait.

## Reading it through the toolchain

[`cargo cgp check`](https://github.com/contextgeneric/cargo-cgp) rewrites recognized CGP diagnostics
with readable field names and dependency chains. For this missing-field case, its abbreviated output
identifies both the unavailable trait and its cause:

```text
error[E0277]: [CGP-E001] missing field `sent_emails` on `BrokenApp`
   |
   = note: root cause: [CGP-E110] missing field `sent_emails` on `BrokenApp`
           this is required through the dependency chain:
             [CGP-E101] consumer trait impl `CanSendEmail` for context `BrokenApp`
             └─ [CGP-E102] provider trait impl `EmailSender` with context `BrokenApp`
                           for provider `RecordEmails`
               └─ [CGP-E110] missing field `sent_emails` on `BrokenApp`
```

Here the dependency chain connects the application's need for `CanSendEmail` to the provider it
selected and then to the missing `sent_emails` field. This gives the reader both the cause and the
reason that field is required.

The tool can also recover causes that ordinary compiler output omits. It uses the compiler's
next-generation trait solver for diagnostic recovery, then translates the recognized CGP structures.
`cargo cgp check` leads with the root cause for the classes it recognizes, and the tool does not yet
reshape every class; [Reading the output](/docs/cargo-cgp/reading-output#errors-that-pass-through-unchanged)
lists the ones that pass through unchanged. Output details depend on the compiler and tool version.

## Checking a stack one layer at a time

Checks on individual providers help distinguish a wrapper's requirements from its inner provider's
requirements. Suppose `RectangleArea` needs `width` and `height`, while
`ScaledArea<RectangleArea>` also needs a scale factor. Both can be checked against `App`:

```rust
check_components! {
    #[check_providers(
        RectangleArea,
        ScaledArea<RectangleArea>,
    )]
    App {
        AreaCalculatorComponent,
    }
}
```

If `App` lacks `width`, both assertions fail because the wrapper needs its inner provider to
work. If it lacks only the scale factor, the rectangle assertion passes and the wrapper assertion
fails. Giving each layer its own diagnostic location makes this distinction easier to see when
debugging a [higher-order provider](./higher-order-providers.md).

## What to check, and when to fuse it

Placing checks beside each context's wiring catches missing entries and dependencies before a
caller happens to need them. For basic `Component: Provider` entries, the checks can be part of the
same declaration.

[`delegate_and_check_components!`](/docs/reference/macros/delegate_and_check_components) combines
wiring and checking for basic `Component: Provider` entries. For generic components, its
`#[check_params(...)]` attribute supplies parameter sets to check. Keeping the assertion beside the
entry helps prevent the two from drifting apart.

Separate checks are needed for advanced mappings and individual provider layers. The combined
macro does not derive checks for `open` statements, namespace joins, or path-based entries; those
forms can remain unchecked even inside a combined table. Use `check_components!` to name the
components and parameters you need to verify, and `#[check_providers]` to inspect provider layers.

Check an [aggregate provider](./aggregate-providers.md) through a context that uses it. Applying
the combined macro to the bundle checks the bundle as a context instead. It can pass when the
providers need nothing from their context, or fail because the bundle lacks fields the actual
context supplies. Neither result verifies the intended application.

## What it costs

An explicit check proves only the components and parameter sets it names. As wiring grows, the
checks need to grow with it, or an omitted component can still fail at a later call site. For a
generic component, this may mean checking several concrete parameter sets that the application
relies on.

Moving the check does not necessarily shorten its diagnostic. Several broken entries can produce
several errors, and a shared missing dependency may appear once for every provider that needs it.
`cargo cgp check` makes the cases it recognizes easier to follow, while raw compiler output can
remain verbose.

Even a successful check establishes only the trait requirements. A provider may depend on an
ordinary bound such as `Value: Display`, which still needs an ordinary Rust implementation. Without
the corresponding CGP machinery, that trait cannot itself be listed as a component to check.
And satisfying these bounds says nothing about whether the email body was recorded correctly;
that behavior still needs tests.

## Where to go next

These pages explain the dependencies being checked and the available check forms:

- [Impl-side dependencies](./impl-side-dependencies.md): Why provider requirements can be absent from
  the caller's interface.
- [Higher-order providers](./higher-order-providers.md): Composing providers and checking their layers.
- [Aggregate providers](./aggregate-providers.md): Verifying shared wiring against a real context.
- [`check_components!`](/docs/reference/macros/check_components): Component parameters and
  `#[check_providers]` assertions.
- [`delegate_and_check_components!`](/docs/reference/macros/delegate_and_check_components): Combined
  wiring and checking, including its coverage limits.
- [Comparison: Dependency injection](/docs/comparisons/dependency-injection): `check_components!` beside a container's startup validation and Dagger's build-time check.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
