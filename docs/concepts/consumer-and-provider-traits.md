---
sidebar_label: 'Consumer and provider traits'
sidebar_position: 2
---

# Consumer and provider traits

A production application may send email through SMTP, while a test application records messages
for inspection. Both can expose the same `send_email` method through an ordinary Rust trait.
As more application types need those behaviors, it becomes useful to share each implementation
while letting each application choose which one to use.

CGP supports that choice by separating the interface callers use from the interface reusable
implementations target. This page follows the email example through that split, traces a call
through the generated Rust, and shows how the macros connect the pieces. It closes with the
costs and limits of the arrangement.

## What an ordinary Rust trait does, and where it stops

An ordinary Rust trait already lets the two applications expose the same method with different
behavior. Each application type provides its own implementation of `CanSendEmail`, and callers
use the shared interface:

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

Here, `App` and `TestApp` are the contexts on which `send_email` operates. Each represents an
application and carries the data its implementation needs. The SMTP body is omitted, while the
test implementation records the recipient and message in a vector.

To share each behavior with other application types, it might seem natural to write a
[blanket implementation](/docs/reference/glossary#blanket-implementation). The SMTP implementation
could work for any type with `HasSmtpConfig`, and the recording implementation for any type with
`HasRecordedEmails`. Offering both, however, creates a conflict:

```rust
impl<T: HasSmtpConfig>     CanSendEmail for T { /* over SMTP */ }
impl<T: HasRecordedEmails> CanSendEmail for T { /* record it */ }   // error[E0119]
```

A type could satisfy both bounds, leaving Rust with two implementations of `CanSendEmail` for
that type. The compiler therefore rejects the pair. Helper functions and other composition
techniques can still share the code, with each application connecting its trait implementation
to the behavior it wants. CGP makes that connection through a separate trait for reusable
implementations and a wiring table that selects one.

## Splitting one trait in two

The **consumer trait** keeps the caller's interface, including the original methods and receiver.
The **provider trait** moves the consumer's `Self` into a `Context` parameter. Its methods receive
the context as an explicit argument, leaving the implementing type free to name the behavior.

The signatures below show the split. They omit a generated checking bound on the provider trait,
which the call trace explains later:

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

The implementations can now keep their original context bounds while using separate marker
structs as `Self`. With the method bodies omitted, their declarations look like this:

```rust
pub struct SendViaSmtp;
pub struct RecordEmails;

impl<Context: HasSmtpConfig> EmailSender<Context> for SendViaSmtp { /* ... */ }
impl<Context: HasRecordedEmails> EmailSender<Context> for RecordEmails { /* ... */ }
```

Even if a context satisfies both `HasSmtpConfig` and `HasRecordedEmails`, these implementations
have different `Self` types and can coexist. Each local provider struct also lets its crate
implement a provider trait defined elsewhere. The split therefore follows ordinary Rust coherence
rules, as [Bypassing coherence](./coherence.md) explains.

The same split applies to a component with several methods, associated types, or constants. Those
items can stay together when one provider choice should determine their implementation; this
example uses a single method to make the relationship easier to see.

## A provider is a name, not a value

The structs `SendViaSmtp` and `RecordEmails` serve as names for implementations. They are
zero-sized, and CGP neither constructs values of them nor stores application data in them.
That data stays in the context, while explicit method arguments supply inputs such as the
recipient and message body.

Because the state belongs to the context, it helps to distinguish the two meanings of `Self`
when reading provider code. In [`#[cgp_impl]`](/docs/reference/macros/cgp_impl), the method's
`self` receiver refers to the context, and the macro rewrites references to `Self` to the context
type. The provider's own associated types and constants retain their provider meaning. In a raw
Rust provider implementation, `Self` simply names the provider type following `for`.

## How a call finds its provider

With the behaviors defined, each application needs to choose which provider supplies its
`send_email` method. Wiring records that choice in a table of types that the compiler resolves:

```rust
delegate_components! { App     { EmailSenderComponent: SendViaSmtp } }
delegate_components! { TestApp { EmailSenderComponent: RecordEmails } }
```

The key in both tables is `EmailSenderComponent`, a marker generated alongside the two traits.
Those traits, the marker, and the forwarding implementations form the **component**. The entry
for this key selects `SendViaSmtp` on `App` and `RecordEmails` on `TestApp`.

When a caller writes `app.send_email("a@b.c", "hi")`, the compiler follows `App`'s selection to
`SendViaSmtp` and checks whether that provider can work with `App`. This is static dispatch,
so the running program needs neither a wiring table nor a provider lookup.

To see how method syntax reaches that selection, consider the two generated blanket
implementations. The consumer implementation supplies `CanSendEmail` whenever the context
implements `EmailSender` for itself:

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

Satisfying `Context: EmailSender<Context>` takes us to the second implementation. It gives a
delegating type the provider trait of the type selected by its table. Here, `DelegateComponent`
represents the table entry, and its `Delegate` associated type names the selected provider:

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

For the call on `App`, both `Context` and `Provider` initially stand for `App`. In this
implementation, `Provider` names the type whose table is consulted, so its `Delegate` is
`SendViaSmtp`. As long as `SendViaSmtp` implements `EmailSender<App>`, the delegation gives `App`
its own `EmailSender<App>` implementation, which in turn supplies `CanSendEmail` through the first
blanket implementation. The context stays `App` throughout, so the final provider receives the
same application value on which the caller invoked the method.

The other bound, [`IsProviderFor`](/docs/reference/traits/wiring/is_provider_for), helps check
that the provider's dependencies hold. Provider macros generate an implementation of this trait
with the provider's bounds, and the provider trait requires it as a
[supertrait](/docs/reference/glossary#supertrait). This is the checking bound omitted from the
earlier `EmailSender` declaration. Explicit component checks use it to expose missing
requirements that a consumer-trait error can hide.

The listings use shorter names to keep this connection readable. In an actual expansion, the
parameters are `__Context__` and `__Provider__`, and the delegate is spelled as
`<__Provider__ as DelegateComponent<EmailSenderComponent>>::Delegate`. You can inspect the full
expansion of a concrete component with [`cargo cgp expand`](/docs/cargo-cgp/expand).

## Writing it

The macros generate these connections while letting you write methods from the context's point
of view. `#[cgp_component]` produces the trait pair and forwarding implementations, and
`#[cgp_impl]` lets each provider keep a familiar receiver. For the email example, that gives:

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
The macro turns it into a local binding and removes it from the public signature, so callers
still write `app.send_email(to, body)`. The SMTP body is still a placeholder; the recording
provider stores the message in the test application's vector.

The fields differ between providers: `SendViaSmtp` needs `smtp_server`, while `RecordEmails` needs
`sent_emails`. Because these are [impl-side dependencies](./impl-side-dependencies.md), neither
requirement appears in `CanSendEmail`. A caller can therefore use the selected behavior with
that trait bound alone:

```rust
pub fn send_welcome(app: &impl CanSendEmail, to: &str) {
    app.send_email(to, "Welcome!");
}

let test_app = TestApp {
    sent_emails: RefCell::new(Vec::new()),
};
send_welcome(&test_app, "a@b.c");

assert_eq!(test_app.sent_emails.borrow().as_slice(), ["a@b.c: Welcome!"]);
```

The same `send_welcome` function accepts `App` as well as `TestApp`. Its bound names only
`CanSendEmail`, so each application's wiring supplies the behavior without the function knowing
the provider or its fields.

Because the consumer trait is still an ordinary Rust trait, a context can also implement it
directly and omit providers and wiring, as at the start of the page. This lets an application
adopt components one trait at a time. For a given context, however, a direct implementation would
overlap with a usable wired implementation of the same consumer trait, so it cannot have both.

## What it costs

For an operation with one implementation, the trait pair and wiring may add more machinery than
you need. Use a plain trait or consider [`#[cgp_fn]`](/docs/reference/macros/cgp_fn), which generates
a trait and blanket implementation from a function without component wiring.

When you do use wiring, the compiler checks whether it supplies a usable implementation once
code requires the trait. Until then, a missing entry or unsatisfied provider dependency may remain
undetected. [`check_components!`](/docs/reference/macros/check_components) brings those checks to
a place you choose, usually beside the table. If one fails,
[`cargo cgp check`](/docs/cargo-cgp/check) leads with the root cause for the classes it recognizes,
though it does not yet reshape every class. [Checking your wiring](./check-traits.md) explains
those diagnostics and their limits.

Even with static dispatch, a reader tracing the call has to find the table entry and follow it
to the provider. The split is useful when it lets several contexts reuse a provider or select
different behaviors while a caller such as `send_welcome` keeps the same interface.

## Where to go next

To apply the same separation in your own code, consider the dependencies each provider needs
and the choices each context must make. These pages develop those ideas and show the component
model in practice:

- [Bypassing coherence](./coherence.md): Why distinct provider types permit reusable alternatives.
- [Impl-side dependencies](./impl-side-dependencies.md): How a provider states requirements without
  adding them to the caller's interface.
- [Area calculation tutorial](/docs/tutorials/area-calculation/): Building the same split from plain
  functions and calling providers before introducing wiring.
- [`#[cgp_component]`](/docs/reference/macros/cgp_component),
  [`#[cgp_impl]`](/docs/reference/macros/cgp_impl), and
  [`delegate_components!`](/docs/reference/macros/delegate_components): Exact syntax and generated code.
- [Comparison: Dynamic dispatch](/docs/comparisons/dynamic-dispatch): How compile-time wiring compares with a runtime vtable.
- [Comparison: Policy-based design](/docs/comparisons/policy-based-design): The corresponding split in C++ policy classes and host templates.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
