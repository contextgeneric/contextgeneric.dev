---
sidebar_label: 'Checking your wiring'
sidebar_position: 6
---

# Checking your wiring

CGP wiring records provider choices without checking their requirements immediately. A compile-time
check verifies those requirements at a location you choose and helps expose missing dependencies.
This page follows one broken context from a method-call error through an explicit check and
`cargo cgp check`, then explains what each can and cannot verify.

## A context can be wrong and still compile

A wiring table can name a provider even when the context lacks a field that provider needs.
In this example, `RecordEmails` implements the `EmailSender` component by recording messages in
`sent_emails`, but `BrokenApp` supplies only `smtp_server`:

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

The table compiles because it records only the mapping from `EmailSenderComponent` to `RecordEmails`.
The compiler has not yet been asked to prove that `RecordEmails` can send email for `BrokenApp`.
That proof will fail: the `#[implicit]` argument requires a `sent_emails` field.

This deferred checking lets reusable wiring exist before its final context is known.
An [aggregate provider](./aggregate-providers.md), for example, can group provider choices for
contexts defined elsewhere. Each context must satisfy the providers' requirements when it uses
them, but the bundle does not need to supply those dependencies itself.

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

The diagnostic reports that `BrokenApp` lacks `CanSendEmail` and `EmailSender<BrokenApp>`, but does
not identify the missing `sent_emails` field. The failed requirement is hidden behind the generated
trait implementations. The error location may also be far from the wiring, in another module or crate.

## Asking the question at a line you chose

A check trait forces the compiler to verify a bound where you write its implementation. The basic
technique uses an ordinary Rust [supertrait](/docs/reference/glossary#supertrait) and an empty implementation:

```rust
trait CanUseApp: CanSendEmail {}
impl CanUseApp for App {}
```

The empty implementation compiles only if `App: CanSendEmail` holds. Placing it beside the wiring
catches the failure there, although checking the consumer trait alone can still produce the vague
error shown above.

[`check_components!`](/docs/reference/macros/check_components) generates a check that exposes the
selected provider's requirements more directly:

```rust
check_components! {
    BrokenApp {
        EmailSenderComponent,
    }
}
```

The macro checks `CanUseComponent`, which requires a wiring entry and the selected provider's
dependency bounds for this context. That path makes the missing field visible in the diagnostic:

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

The error now identifies a missing `HasField` implementation and names `RecordEmails` as the
provider requiring it. The expanded field names remain hard to read: `sent_emails` and `smtp_server`
appear as nested character types. The headline also names the generated `CanUseComponent` bound
rather than the public trait.

## Reading it through the toolchain

[`cargo cgp check`](https://github.com/contextgeneric/cargo-cgp) rewrites recognized CGP diagnostics
with readable field names and dependency chains. For this missing-field case, its abbreviated output
identifies both the unavailable trait and its cause:

```text
error[E0277]: [CGP-E001] the consumer trait `CanSendEmail` is not implemented
              for context `BrokenApp`
   |
   = note: root cause: [CGP-E106] missing field `sent_emails` on `BrokenApp`
           this is required through the dependency chain:
             [CGP-E101] consumer trait impl `CanSendEmail` for context `BrokenApp`
             └─ [CGP-E102] provider trait impl `EmailSender` with context `BrokenApp`
                           for provider `RecordEmails`
               └─ [CGP-E106] missing field `sent_emails` on `BrokenApp`
```

The root cause is now explicit: `BrokenApp` lacks `sent_emails`, which `RecordEmails` requires.
The dependency chain connects that missing field to the `CanSendEmail` trait the application needs.

The tool can also recover causes that ordinary compiler output omits. It uses the compiler's
next-generation trait solver for diagnostic recovery, then translates the recognized CGP structures.
Its `v0.1.0-alpha` release covers core wiring errors, but some classes, including orphan-rule
failures, still pass through unchanged. Output details depend on the compiler and tool version.

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

A missing `width` requirement fails both assertions because the wrapper also needs its inner
provider to work. A missing scale factor fails only the wrapper's assertion. Separate diagnostic
locations make that difference easier to see when debugging a
[higher-order provider](./higher-order-providers.md).

## What to check, and when to fuse it

Check each context's required components when you define its wiring. This catches missing entries
and unsatisfied dependencies before a later caller happens to exercise them.

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

Explicit checks cover only the components and parameter sets you list. Add them as you add wiring;
otherwise an omitted component can still fail only when a caller uses it. Generic wiring may need
several checks to cover the concrete uses an application depends on.

A check can expose a cause without making the diagnostic short. Several invalid entries can produce
several errors, and one shared missing dependency can appear in multiple provider failures.
`cargo cgp check` makes recognized cases easier to read, but raw compiler diagnostics remain verbose.

Checks verify trait requirements rather than runtime behavior. Ordinary bounds such as `Value: Display`
can still fail as dependencies of a provider, and those traits need ordinary Rust implementations.
They cannot be listed as components in a wiring check unless they have the corresponding CGP component
machinery. Tests remain necessary for behavior such as whether an email was recorded correctly.

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
