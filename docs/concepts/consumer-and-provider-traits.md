---
sidebar_label: 'Consumer and provider traits'
sidebar_position: 2
---

# Consumer and provider traits

CGP separates the trait callers use from the trait providers implement. A **consumer trait** exposes
a capability on a context, while a **provider trait** lets reusable implementations supply that
capability. This page motivates the split, traces a method call through its wiring, and shows the
generated Rust before discussing the costs.

## What an ordinary Rust trait does, and where it stops

An ordinary Rust trait connects a type to an implementation of a capability. A `T: Display` bound,
for example, lets the compiler select the formatting implementation for `T`. This is sufficient when
each type needs one implementation and you do not need to choose among reusable alternatives.

Separate application types can implement the same trait differently using ordinary Rust.
A production application might send email, while a test application records it:

```rust
use core::cell::RefCell;

pub trait CanSendEmail {
    fn send_email(&self, to: &str, body: &str);
}

pub struct App {
    pub smtp_server: String,
}

pub struct TestApp {
    pub sent_emails: RefCell<Vec<String>>,
}

impl CanSendEmail for App {
    fn send_email(&self, to: &str, body: &str) {
        // open a connection to self.smtp_server and send
    }
}

impl CanSendEmail for TestApp {
    fn send_email(&self, to: &str, body: &str) {
        self.sent_emails.borrow_mut().push(format!("{to}: {body}"));
    }
}
```

`App` and `TestApp` already provide different behavior without CGP. Each is a context representing
an application and carrying the data its implementation needs. The SMTP body is omitted here;
`TestApp` shows the recording behavior in full.

Overlapping blanket implementations prevent those behaviors from being offered as interchangeable
trait implementations. If the SMTP implementation applies to every type with `HasSmtpConfig`, and
the recording implementation applies to every type with `HasRecordedEmails`, they conflict:

```rust
impl<T: HasSmtpConfig>     CanSendEmail for T { /* over SMTP */ }
impl<T: HasRecordedEmails> CanSendEmail for T { /* record it */ }   // error[E0119]
```

A type could satisfy both bounds, so Rust rejects the pair. The bodies can still be shared through
helper functions or other composition techniques, but each application must connect its trait
implementation to the chosen behavior. CGP provides a reusable provider interface and wiring for
that choice.

## Splitting one trait in two

CGP gives callers and implementers separate trait interfaces:

- The **consumer trait** keeps the original methods, so callers use `app.send_email(..)`.
- The **provider trait** moves the consumer's `Self` into an explicit `Context` parameter and
  replaces the receiver with a context argument.

The simplified pair for email sending looks like this. The provider's generated checking bound is
omitted here and explained below:

```rust
// what callers use
pub trait CanSendEmail {
    fn send_email(&self, to: &str, body: &str);
}

// what implementations target
pub trait EmailSender<Context> {
    fn send_email(context: &Context, to: &str, body: &str);
}
```

The provider trait lets a named implementation supply behavior for a context. Each implementation
uses its own marker type in the `Self` position:

```rust
impl<Context> EmailSender<Context> for SendViaSmtp { /* ... */ }
impl<Context> EmailSender<Context> for RecordEmails { /* ... */ }
```

The implementations have distinct `Self` types, so they can accept the same contexts without
conflicting with one another. Owning the provider type also lets a crate implement a provider trait
from another crate. These are ordinary coherent Rust implementations;
[Bypassing coherence](./coherence.md) explains the rules behind the arrangement.

A component can contain several methods, associated types, and constants. The single-method example
keeps the split visible; it is not a restriction imposed by CGP. Group items when one provider choice
should determine their implementation.

## A provider is a name, not a value

A provider such as `SendViaSmtp` or `RecordEmails` is a zero-sized type used to name an implementation.
CGP does not construct a provider value or store state in it. The context supplies the values the
implementation needs.

Within [`#[cgp_impl]`](/docs/reference/macros/cgp_impl), `self` and `Self` refer to the context.
The macro rewrites them into the explicit context argument and type parameter shown above.
In a raw Rust provider implementation, `Self` instead has its ordinary meaning: the provider type
following `for`.

## How a call finds its provider

A context's wiring selects a provider for each component, much like a settings table whose keys and
values are types. The compiler resolves this table at compile time:

```rust
delegate_components! { App     { EmailSenderComponent: SendViaSmtp } }
delegate_components! { TestApp { EmailSenderComponent: RecordEmails } }
```

`EmailSenderComponent` is the generated marker used as the key. A **component** groups the consumer
trait, provider trait, and that marker. The wiring maps its key to `SendViaSmtp` for `App` and to
`RecordEmails` for `TestApp`.

A method call follows the selected entry. For `app.send_email("a@b.c", "hi")`, the compiler resolves
`App`'s email-sending capability through its table to `SendViaSmtp`, then checks that provider's
requirements against `App`. The call uses static dispatch; the compiled program does not need a
runtime wiring table or provider lookup.

The generated consumer implementation connects method syntax to the provider interface. Any context
that implements the provider trait for itself gets the consumer trait:

```rust
impl<Context> CanSendEmail for Context
where
    Context: EmailSender<Context>,
{
    fn send_email(&self, to: &str, body: &str) {
        Context::send_email(self, to, body)
    }
}
```

The generated delegation implementation then connects a table entry to its selected provider.
`DelegateComponent` represents that entry, and its `Delegate` associated type names the provider:

```rust
impl<Context, Provider> EmailSender<Context> for Provider
where
    Provider: DelegateComponent<EmailSenderComponent>
        + IsProviderFor<EmailSenderComponent, Context, ()>,
    Provider::Delegate: EmailSender<Context>,
{
    fn send_email(context: &Context, to: &str, body: &str) {
        Provider::Delegate::send_email(context, to, body)
    }
}
```

Together, these implementations resolve `App: CanSendEmail` through `App: EmailSender<App>` to
`SendViaSmtp: EmailSender<App>`. The context remains `App` throughout, so the final provider receives
the application value on which the caller invoked the method.

[`IsProviderFor`](/docs/reference/traits/wiring/is_provider_for) carries the provider's dependency
bounds for checking. The provider macros generate an implementation with those bounds, and the
provider trait also requires it as a supertrait. Explicit component checks use this path to expose
missing requirements that a consumer-trait error can hide.

The listings simplify generated names for readability. Actual expansions use `__Context__` and
`__Provider__`, and spell the delegate as
`<__Provider__ as DelegateComponent<EmailSenderComponent>>::Delegate`. The earlier simplified
`EmailSender` declaration also omitted its `IsProviderFor` supertrait. `cargo cgp expand` shows the
full expansion for a concrete example.

## Writing it

The macros generate the trait pair and forwarding implementations from a consumer trait definition.
`#[cgp_component]` creates the component, and `#[cgp_impl]` defines each provider using context-style
method syntax:

```rust
use cgp::prelude::*;
use core::cell::RefCell;

#[cgp_component(EmailSender)]
pub trait CanSendEmail {
    fn send_email(&self, to: &str, body: &str);
}

#[cgp_impl(new SendViaSmtp)]
impl EmailSender {
    fn send_email(&self, #[implicit] smtp_server: &str, to: &str, body: &str) {
        // open a connection to `smtp_server` and send the message
    }
}

#[cgp_impl(new RecordEmails)]
impl EmailSender {
    fn send_email(&self, #[implicit] sent_emails: &RefCell<Vec<String>>, to: &str, body: &str) {
        sent_emails.borrow_mut().push(format!("{to}: {body}"));
    }
}

#[derive(HasField)]
pub struct App {
    pub smtp_server: String,
}

#[derive(HasField)]
pub struct TestApp {
    pub sent_emails: RefCell<Vec<String>>,
}

delegate_components! { App     { EmailSenderComponent: SendViaSmtp } }
delegate_components! { TestApp { EmailSenderComponent: RecordEmails } }
```

Each `#[implicit]` argument reads a same-named field exposed by the context's `HasField` derive.
The macro removes these arguments from the public signature, so callers still write
`app.send_email(to, body)`. The SMTP body remains a placeholder; the recording provider stores the
message in the test application's vector.

Each provider declares only the fields it needs. `SendViaSmtp` requires `smtp_server`, while
`RecordEmails` requires `sent_emails`; neither requirement appears in `CanSendEmail`.
These are [impl-side dependencies](./impl-side-dependencies.md), which let implementations have
different requirements without changing the caller's interface.

A context can also implement the consumer trait directly, using the ordinary Rust form shown at
the start of the page. For that capability it can omit providers and wiring. This allows adoption
one capability at a time, subject to Rust's usual restriction against conflicting implementations.

## What it costs

A component adds declarations and wiring beyond a plain trait. For a capability with one
implementation, use a plain trait or consider [`#[cgp_fn]`](/docs/reference/macros/cgp_fn), which
creates a blanket-implemented capability from a function without component wiring.

Wiring does not immediately verify the selected provider's requirements. A missing entry or
unsatisfied dependency can remain undetected until a caller needs the capability.
[`check_components!`](/docs/reference/macros/check_components) verifies the requirements where you
place the check, usually beside the table.
[`cargo cgp check`](https://github.com/contextgeneric/cargo-cgp) reports recognized failures with
readable causes; its `v0.1.0-alpha` release covers core wiring errors rather than every class.
[Checking your wiring](./check-traits.md) shows what to expect with and without those tools.

Tracing a method call requires following the wiring to its provider. Static dispatch avoids runtime
lookup costs, but a reader still has to find the table entry and the implementation it names.
The separation pays for itself when those implementations are reused or selected differently across
contexts.

## Where to go next

These pages develop the trait model and show how to use it:

- [Bypassing coherence](./coherence.md): Why distinct provider types permit reusable alternatives.
- [Impl-side dependencies](./impl-side-dependencies.md): How a provider states requirements without
  adding them to the caller's interface.
- [Area calculation tutorial](/docs/tutorials/area-calculation/): Building the same split from plain
  functions and calling providers before introducing wiring.
- [`#[cgp_component]`](/docs/reference/macros/cgp_component),
  [`#[cgp_impl]`](/docs/reference/macros/cgp_impl), and
  [`delegate_components!`](/docs/reference/macros/delegate_components): Exact syntax and generated code.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
