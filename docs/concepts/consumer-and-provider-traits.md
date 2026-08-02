---
sidebar_label: 'Consumer and provider traits'
sidebar_position: 2
---

# Consumer and provider traits

One trait definition in CGP becomes two traits: a **consumer trait** that callers use, and a
**provider trait** that implementations target. This page explains why the split exists, what each
half is for, and how a plain method call finds its way from one to the other. It closes on what the
arrangement costs, which is the part worth reading before adopting it.

## What an ordinary Rust trait does, and where it stops

A Rust trait joins two jobs that are usually the same job. The type that implements `Display` is the
type callers format, and that identity is what lets the compiler resolve a `where T: Display` bound
without anyone naming an implementation — it looks the type up, finds the one implementation, and is
done. Almost all Rust code wants exactly this, which is why the language is built around it.

The consequence is that a trait offers one implementation per type, and that limit shows up sooner
than it sounds. Consider an application that sends email, with a test harness that must not:

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

This compiles, and it is worth noticing that it already does what you want: two application types,
two behaviours, no CGP anywhere. `App` and `TestApp` here are types that stand for a whole
application — their job is to carry the choices and the data the program needs, not to be operated
on. That shape is legal Rust and nothing about it is unusual.

What fails is *sharing*. As soon as you try to lift a body out into something reusable — so that a
second application could adopt the SMTP behaviour without copying it — the two implementations
collide:

```rust
impl<T: HasSmtpConfig>     CanSendEmail for T { /* over SMTP */ }
impl<T: HasRecordedEmails> CanSendEmail for T { /* record it */ }   // error[E0119]
```

The compiler rejects the second impl because a type could satisfy both bounds, and it has no
principled way to choose. So the arrangement above survives, but every application must hand-write
every body, and nothing can be factored out. It is available and unrewarding, which is why almost
nobody builds on it — and it is exactly the gap CGP fills.

## Splitting one trait in two

CGP's move is to stop implementing the trait for the type the capability is about. A single
definition produces two traits with the two jobs pulled apart:

- The **consumer trait** is the caller's view. It keeps the original name, the original `self`
  receiver, and the original signatures, so a caller writes `app.send_email(..)` as before.
- The **provider trait** is the implementer's view. It is the same interface with `Self` moved out
  into an explicit leading `Context` type parameter, and every `self` rewritten to `context`.

For the example above, the pair is:

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

Moving `Self` is the whole trick. An implementation no longer says "this type sends email"; it says
"this *named implementation* sends email, for any context". The name is a small type the
implementing crate declares for the purpose:

```rust
impl<Context> EmailSender<Context> for SendViaSmtp { /* ... */ }
impl<Context> EmailSender<Context> for RecordEmails { /* ... */ }
```

Both compile, and any number more would. There is no overlap, because each `impl` has a different
`Self`. There is no orphan-rule problem either, because that `Self` is a type the crate writing the
impl owns — which means a crate can add behaviour for a type it did not define. Neither rule was
repealed; the implementations simply stopped being the kind of thing the rules are about. The wider
argument for why that is a fair trade is [Bypassing coherence](./coherence.md).

## A provider is a name, not a value

A **provider** is one of those named implementations — `SendViaSmtp`, `RecordEmails`. It is a
zero-sized type that exists only to be named. Nothing constructs it, nothing stores it, and there is
no field in it to read; at runtime it does not exist at all.

That has one consequence worth fixing in mind before reading any provider code. Inside a provider,
`self` and `Self` refer to the **context** — the application type the method is running against —
never to the provider. When you write a provider with
[`#[cgp_impl]`](/docs/reference/macros/cgp_impl), the macro keeps `self` in the source and rewrites
it to the context underneath, precisely because the context is the only value that exists when the
method runs.

## How a call finds its provider

Two pieces connect the halves: a table on the context that names its choice, and a pair of generated
implementations that follow it.

The table is what a context writes. It maps each **component** — one capability, defined once, that
implementations can be wired for — to the provider that should supply it, and it is the one place a
choice is recorded:

```rust
delegate_components! { App     { EmailSenderComponent: SendViaSmtp } }
delegate_components! { TestApp { EmailSenderComponent: RecordEmails } }
```

`EmailSenderComponent` is a marker type generated alongside the two traits, used purely as the key.
The table itself is a set of trait implementations rather than a runtime structure — the lookup is
performed by the compiler, and nothing survives into the program.

The two generated implementations then chain through it. The first says that any context which
implements the *provider* trait for itself gets the *consumer* trait, which is what makes
`app.send_email(..)` a legal call:

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

The second says that anything with a table entry for this component inherits the provider trait from
whatever the entry names:

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

Put together, `app.send_email("a@b.c", "hi")` resolves in three steps. `App` implements
`CanSendEmail` because it implements `EmailSender<App>` for itself; it implements `EmailSender<App>`
because its table maps `EmailSenderComponent` to `SendViaSmtp`; and `SendViaSmtp` implements
`EmailSender<App>` directly. Every step is trait resolution, so the finished call is a direct call to
`SendViaSmtp::send_email` — no lookup happens while the program runs, and a provider no context uses
never reaches the binary.

The one piece of that listing not yet explained is
[`IsProviderFor`](/docs/reference/traits/is_provider_for), which also rides on the provider trait as a
supertrait. Every provider implements it under exactly the bounds it needs, and requiring it here is
what carries those bounds back down the chain — so when a context is missing something a provider
requires, the compiler can name the missing requirement instead of reporting only that the provider
trait is not implemented. It is generated, never written by hand.

Three liberties in the listings above are worth knowing before you read a real error message. The
generated type parameters carry reserved names — the context is `__Context__` and the provider
`__Provider__` — and the delegate is spelled out in full as
`<__Provider__ as DelegateComponent<EmailSenderComponent>>::Delegate`. `Context`, `Provider`, and
`Provider::Delegate` here are for legibility. And the provider trait printed earlier on this page omitted
its `IsProviderFor` supertrait, which the real `EmailSender<__Context__>` carries. `cargo cgp expand` will
show you all of it as the macros actually emit it.

## Writing it

In practice you write neither trait by hand. [`#[cgp_component]`](/docs/reference/macros/cgp_component)
generates the pair, the marker, and the two blanket implementations from one trait definition, and
[`#[cgp_impl]`](/docs/reference/macros/cgp_impl) lets a provider be written in the consumer trait's
shape — keeping `self` and the original signatures — and rewrites it into the provider form:

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

Two details of that listing are worth pointing out. The
[`#[implicit]`](/docs/reference/attributes/implicit) arguments are how each provider reaches into its
context: they read a same-named field and disappear from the method's public signature, so
`app.send_email(to, body)` still takes two arguments. And the two providers depend on *different*
fields — `SendViaSmtp` needs an `smtp_server`, `RecordEmails` needs somewhere to record — with
neither requirement appearing in `CanSendEmail`. That is
[impl-side dependency injection](./impl-side-dependencies.md), and it is what stops a context paying
for the needs of implementations it did not choose.

A consumer trait remains an ordinary Rust trait throughout. Nothing stops a context from implementing
it directly, exactly as in the very first listing on this page, and skipping providers and wiring
entirely. The split is something you opt into for the capabilities that need more than one
implementation, not a replacement for how traits already work.

## What it costs

**It is more machinery than a plain trait.** One capability becomes two traits, a marker type, and a
line of wiring per context. For a capability with a single implementation, that is pure overhead —
write a plain trait, or reach for [`#[cgp_fn]`](/docs/reference/macros/cgp_fn), which builds a
capability from a function with no component and no wiring at all.

**Wiring is checked lazily.** A table with a missing entry, or one naming a provider whose own
requirements the context cannot meet, still compiles; the failure surfaces later, wherever the
capability is finally used, and the error can be long. This is real, and it has an answer:
[`check_components!`](/docs/reference/macros/check_components) forces the check at the wiring line so
the error names the actual gap, and
[`cargo cgp check`](https://github.com/contextgeneric/cargo-cgp) reshapes the recognized failures to
lead with the root cause, though it is a `v0.1.0-alpha` and covers the core wiring errors rather than
every class. Neither makes the raw diagnostics pleasant — see
[Checking your wiring](./check-traits.md) for what to expect.

**There is a hop between the call and the code that runs.** `app.send_email(..)` no longer points at
one body you can jump to. Unlike runtime dispatch the indirection is fully resolved at compile time
and never ambiguous, and the wiring table is a single greppable place naming exactly one provider per
component — but it is a hop, and a reader unfamiliar with the codebase has to follow it.

## Where to go next

[Bypassing coherence](./coherence.md) is the argument underneath this page: why Rust's rule exists,
why it is correct, and why scoping it per context is a fair trade rather than a loophole.
[Impl-side dependencies](./impl-side-dependencies.md) develops the other half of what makes providers
reusable — how an implementation states what it needs without the interface carrying it.

To write this rather than read about it, the [Area calculation tutorial](/docs/tutorials/area-calculation/)
builds the same split up from plain functions, and calls providers by name before any wiring exists.
For the exact syntax and the full generated code, the reference pages for
[`#[cgp_component]`](/docs/reference/macros/cgp_component),
[`#[cgp_impl]`](/docs/reference/macros/cgp_impl), and
[`delegate_components!`](/docs/reference/macros/delegate_components) are the complete account.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
