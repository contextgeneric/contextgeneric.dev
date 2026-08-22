---
sidebar_label: 'Checking your wiring'
sidebar_position: 6
---

# Checking your wiring

Why a wiring mistake still compiles, and how a compile-time assertion turns the resulting error into
one that names the real cause.

This page answers *why did my mistake not fail where I made it?* It follows one broken context through
three stages, unchecked, checked, and checked through the error toolchain, quoting what the compiler
actually reports at each. It closes on what checking does not fix, which is the honest version of CGP's
most-cited cost.

## A context can be wrong and still compile

Wiring records a choice. It does not verify one.

Here is an application wired to a provider that records outgoing email, and an application that has
nowhere to record it:

```rust
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

`RecordEmails` needs a `sent_emails` field. `BrokenApp` has a `smtp_server`. **This compiles**, and it
is meant to: the table stores "this key points to this provider" as a type-level fact, and nothing has
asked yet whether the provider's own requirements hold for this particular context.

That laziness is not an oversight. It lets a provider be written once against every possible
context, lets a bundle of wiring be reused by applications the bundle has never heard of, and lets a
table be assembled from pieces that never meet. Checking each entry as it was written would mean every
entry knowing its final context, which is the coupling the whole design exists to avoid.

The price is that a context can look finished and be broken.

## Where the failure surfaces instead

The compiler asks the question the first time something uses the capability, which may be in another
module, another crate, or a test somebody runs next week. And the answer arrives in a form that does
not name the problem:

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

Read it closely and it says only that the capability is unavailable, restated twice. The word
`sent_emails` does not appear. Neither does anything about a field. The compiler answered the question
it was asked, *does this type have this method?*, and discarded the reasoning that produced "no" on the
way out.

This is the single worst experience CGP offers a newcomer, and it is worth being blunt that a mis-wire
looks like this by default.

## Asking the question at a line you chose

A **check** forces the same question early, at the wiring, where you can see it. It is not a new
mechanism. The plain-Rust form is a trait that demands something and an impl with nothing in it:

```rust
trait CanUseApp: CanSendEmail {}
impl CanUseApp for App {}
```

The impl has nothing to prove on its own, so it compiles exactly when `App: CanSendEmail` holds and
fails otherwise. Put it beside the table and a latent gap becomes an error on a known line.

[`check_components!`](/docs/reference/macros/check_components) writes that for you, from a list of the
components to verify:

```rust
check_components! {
    BrokenApp {
        EmailSenderComponent,
    }
}
```

But the important part is not the convenience. A check asserts something *stronger* than the consumer
trait, and that stronger assertion changes the error. Asking "does `BrokenApp` implement
`CanSendEmail`?" gets the answer above. Asking "can `BrokenApp` use this component?" makes the compiler
evaluate the provider's actual requirements and report the one that failed:

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

The cause is in there now. A field is missing, one *is* present, and the provider that wanted it is
named. Two things still stand between that and a usable message: the field names are spelled as
type-level character lists, `sent_emails` and `smtp_server` at one character per layer, and the headline
is about a trait nobody wrote.

## Reading it through the toolchain

[`cargo cgp check`](https://github.com/contextgeneric/cargo-cgp) runs in place of `cargo check` and
reshapes the classes it recognizes. On the same code:

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

Missing field `sent_emails` on `BrokenApp`, and the path that wanted it. That is the mistake, in the
words you would use to describe it.

The tool does more than reformat: for the worst class it turns on the compiler's next-generation trait
solver to recover a cause the default solver discards entirely, which is why the first error on this
page had nothing to reshape. It is a `v0.1.0-alpha`, it covers the core wiring errors rather than every
class, and some still pass through as the compiler wrote them, orphan-rule failures among them.
Dramatically better and actively improving, not solved.

## Checking a stack one layer at a time

When providers are built from other providers, checking the context tells you *something* is broken and
not which layer. A variant of the check asserts against each provider rather than against the context:

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

Each provider is now verified on its own line. A requirement missing from the inner `RectangleArea`
fails on both lines, since the outer one needs it too; one missing only from the wrapper fails on the
wrapper alone. The difference between those two shapes is how you find the layer at fault, and it is the
practical tool for debugging a [higher-order provider](./higher-order-providers.md).

## What to check, and when to fuse it

The rule that does not bend is that a context's wiring is checked **somehow**. Which macro does it
scales with how complicated the wiring is.

For a starter context, or a table of plain `Component: Provider` entries,
[`delegate_and_check_components!`](/docs/reference/macros/delegate_and_check_components) does both at
once, so the check cannot be forgotten and cannot drift from the table. Its reach stops at that basic
form: it cannot derive checks for per-type dispatch, for namespaced wiring, or for individual provider
layers, because those need parameters or provider names it has no way to infer.

For anything past that, keep the two apart. A standalone check is where concrete parameters for a
generic component go, where `#[check_providers]` goes, and where opened or namespaced wiring is
verified.

One case is not a matter of taste: **never fuse the check onto a
[bundle of wiring](./aggregate-providers.md)**. A bundle is a provider other contexts delegate to, not a
context, so a context-side check on it asks a question it was never meant to answer, and gets an answer
that means nothing either way. It passes vacuously when the bundled providers need nothing from their
context, and fails blaming the bundle when any of them does. Verify a bundle through a context that
delegates to it.

## What it costs

**A check is something you write.** Nothing generates it from the table unless you fuse the two, so a
component nobody listed is a component nobody verified. In practice the discipline is to add the check
when the context is created, not when it breaks.

**It moves the error; it does not shrink it.** The output above is better because it names the cause,
not because it is short. A context wired wrong in several places reports several failures, and one deep
mistake reaching many providers can report at each of them.

**A check verifies wiring, not everything.** Some of what a provider needs is an ordinary Rust trait
rather than a CGP component, and a bound like `Value: Display` failing looks like any other unsatisfied
bound. There is nothing CGP-specific to route it through.

**And the raw diagnostics remain verbose** for anyone not running the toolchain, which is why this page
quotes all three stages rather than only the last. A reader who meets stage one with no idea stages two
and three exist is the reader CGP loses.

## Where to go next

[Impl-side dependencies](./impl-side-dependencies.md) is why the requirement was hidden in the first
place, the design decision this page pays for. [Higher-order providers](./higher-order-providers.md) is
where per-layer checking earns its keep, and [Aggregate providers](./aggregate-providers.md) is the case
where the context-side check is the wrong one.

For the constructs, [`check_components!`](/docs/reference/macros/check_components) carries every form
including `#[check_providers]` and per-component parameters, and
[`delegate_and_check_components!`](/docs/reference/macros/delegate_and_check_components) is the fused
version for basic wiring.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
