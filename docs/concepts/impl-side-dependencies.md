---
sidebar_label: 'Impl-side dependencies'
sidebar_position: 3
---

# Impl-side dependencies

Declaring what an implementation needs in the implementation itself, so the requirement never
reaches the callers of the interface.

This page answers *how does an implementation say what it needs, without every caller having to know?*
It starts from a technique already available in ordinary Rust, shows why CGP is built on it, and works
through the three kinds of thing an implementation can ask its context for. It closes on what the
arrangement costs, which is mostly paid in error messages.

## What a `where` clause costs the callers above it

A generic function that needs something says so in its signature, and that is the whole difficulty:

```rust
pub trait HasName {
    fn name(&self) -> &str;
}

pub fn greet<Context>(context: &Context) -> String
where
    Context: HasName,
{
    format!("Hello, {}!", context.name())
}
```

Nothing is wrong with this until something calls it. A caller that is itself generic cannot satisfy
`Context: HasName` on its own, so it has to demand the same thing from *its* caller:

```rust
pub fn greet_twice<Context>(context: &Context) -> String
where
    Context: HasName,
{
    format!("{} {}", greet(context), greet(context))
}
```

`greet_twice` never touches a name. It declares the requirement because the function it calls does, and
the function above `greet_twice` will declare it for the same reason. Add a requirement four layers
down and every layer above grows a bound it has no interest in — which is why a mature generic API
tends to accumulate signatures nobody can read, and why adding a dependency deep in a library is a
change that reaches its users.

## Moving the requirement onto the implementation

Rust already has the answer, and most Rust programmers have used it without naming it. Put the logic in
a **blanket implementation** — one `impl` covering every type that meets a bound — and the requirement
moves out of the interface and onto the implementation:

```rust
pub trait CanGreet {
    fn greet(&self) -> String;
}

impl<Context> CanGreet for Context
where
    Context: HasName,
{
    fn greet(&self) -> String {
        format!("Hello, {}!", self.name())
    }
}
```

`CanGreet` mentions nothing. Any type with a `name` gets `greet` for free, and a caller says only what
it actually uses:

```rust
pub fn greet_twice<Context>(context: &Context) -> String
where
    Context: CanGreet,
{
    format!("{} {}", context.greet(), context.greet())
}
```

The requirement did not disappear — the compiler still checks it, at the point where a concrete type
meets the impl. What changed is that it stopped being part of the contract. That is an **impl-side
dependency**: something an implementation needs, stated where the implementation lives rather than
where the interface is declared.

This is the construct CGP is built out of, which is worth saying plainly, because it makes the
foundation something you already have. If you have used `Itertools` or `StreamExt`, you have used a
blanket impl over every `Iterator` or every `Stream` — that is why a method appears on a type whose
author never wrote it. What CGP adds is not the mechanism but the ability to have more than one of them
and choose between them.

## Two providers, two sets of requirements, one interface

The payoff arrives once a capability has more than one implementation, because each one asks for
something different and the interface still says nothing.

Two ways of sending email need entirely different things from the application. One needs a server
address; the other needs somewhere to record what it would have sent:

```rust
#[cgp_component(EmailSender)]
pub trait CanSendEmail {
    fn send_email(&self, to: &str, body: &str);
}

#[cgp_impl(new SendViaSmtp)]
#[uses(HasSmtpServer)]
impl EmailSender {
    fn send_email(&self, to: &str, body: &str) {
        // open a connection to `self.smtp_server()` and send
    }
}

#[cgp_impl(new RecordEmails)]
#[uses(HasSentEmails)]
impl EmailSender {
    fn send_email(&self, to: &str, body: &str) {
        self.sent_emails().borrow_mut().push(format!("{to}: {body}"));
    }
}
```

`#[uses(...)]` is how a provider declares a capability it needs. It reads like an import, and that is
the right way to take it: the provider is saying *this code relies on the context being able to do
this*. Underneath it becomes a bound on the implementation, exactly as in the blanket impl above.

The consequence is worth stating precisely. `CanSendEmail` has no idea that SMTP servers exist. A
function bounded on it accepts both applications, even though they satisfy it for reasons that have
nothing in common:

```rust
pub fn notify<Context>(context: &Context)
where
    Context: CanSendEmail,
{
    context.send_email("a@b.c", "hi");
}
```

Both examples here wire a type standing for an application — `App` for production, `TestApp` for a test
harness — rather than a piece of data. That is where most CGP code lives, and it is what makes the
point land: the two applications differ in what they can supply, and the interface between them and
`notify` does not record the difference.

## The three things an implementation can ask for

Everything above is one kind of requirement — a capability. There are three, they all work the same
way, and each has a syntax that keeps the bound out of sight.

**A capability** is what `#[uses]` declares, as above. It covers other CGP capabilities and ordinary
Rust traits alike: `#[uses(AsRef<[u8]>)]` is as valid as `#[uses(HasSmtpServer)]`.

**A value** is a field the implementation reads, and it is declared by writing it as an argument:

```rust
#[cgp_impl(new GreetByName)]
impl Greeter {
    fn greet(&self, #[implicit] name: &str) -> String {
        format!("Hello, {name}!")
    }
}
```

The `name` argument is not passed by the caller. It is read from a `name` field on the context, the
requirement is generated from the argument's name and type and lands on the implementation, and
`greet()` still takes nothing from the outside. [Implicit arguments](./implicit-arguments.md) develops
this one.

**A type** is the third, and it is the one that buys the most, because what it displaces is not a
leaked bound but a leaked *parameter*.

## Type dependencies, and why they need no parameter

Suppose a job runner should not commit to an error type, a runtime, or a storage backend. The ordinary
way to leave a type open is a generic parameter, and a parameter is an **input**: the caller supplies
it, so it has to appear in the signature.

```rust
pub fn run_job<E, R, S>(store: &S, runtime: &R) -> Result<(), E>
where
    E: From<S::Error> + From<R::Error>,
    R: Runtime,
    S: Store,
{
    // This layer touches none of the three.
}
```

This is the leak from the first section, and worse, because a parameter *cannot* be moved onto an impl.
It is part of the interface by construction. Every intermediate function that merely passes a value
along declares all three and repeats their bounds, and adding a fourth open type is a breaking change
for every caller.

An **abstract type** inverts the direction. Rather than the caller supplying the type, the context
determines it — it is an associated type on a trait the context implements, so an implementation can
name it without anyone choosing it at a call site:

```rust
#[cgp_component(JobRunner)]
#[use_type(HasErrorType.Error)]
pub trait CanRunJob {
    fn run_job(&self) -> Result<(), Error>;
}
```

`#[use_type(HasErrorType.Error)]` imports the context's error type and lets the signature name it as a
bare `Error`. The intermediate layer then has nothing to declare at all:

```rust
#[cgp_fn]
#[uses(CanRunJob)]
#[use_type(HasErrorType.Error)]
pub fn run_jobs(&self) -> Result<(), Error> {
    self.run_job()?;
    self.run_job()
}
```

A parameter is an input and propagates everywhere; an abstract type is an output and propagates
nowhere. That asymmetry is why a CGP codebase can keep accumulating type dependencies without its
signatures growing: **the number of types a context decides can rise freely, because deciding is not
passing.** [Abstract types](./abstract-types.md) is the page for that half.

One thing to be accurate about: when a capability's own signature names the type, the owning trait does
become part of the contract — `CanRunJob` really does imply `HasErrorType`. But a caller bounding on
`CanRunJob` gets that implication for free, never restates it, and names the type only if it handles
one. Compare `trait CanRunJob<E>`, which forces `<E>` onto every caller and every caller's caller
whether they touch an error or not. The bound is on the implementation; the parameter would be on
everyone.

## What it costs

**A hidden requirement is hidden from you too.** The point of all this is that `CanSendEmail` does not
say what its implementations need — which also means reading the interface tells you nothing about what
a context must supply. The answer is in the provider, and finding it means knowing which provider the
context wired.

**The requirement is checked late.** Nothing verifies that a context can satisfy the provider it named
until something uses the capability, so a context can be wired wrong and still compile. That is
[lazy wiring](./check-traits.md), and it is the source of CGP's least pleasant errors.

**When it does fail, the error is about a bound you did not write.** A missing `name` field surfaces as
an unsatisfied field bound naming a type-level spelling of the field name, several layers from the line
that caused it. [`check_components!`](/docs/reference/macros/check_components) forces the failure to the
wiring line and names the actual gap, and
[`cargo cgp check`](https://github.com/contextgeneric/cargo-cgp) leads with the root cause for the
classes it recognizes — a `v0.1.0-alpha` covering the core wiring errors rather than all of them. Both
help substantially; neither makes the raw output pleasant.

**And it is more machinery than a plain function needs.** For a capability with one implementation, the
blanket impl in the second section is the whole of what is useful here, and it is ordinary Rust.
[`#[cgp_fn]`](/docs/reference/macros/cgp_fn) writes exactly that from a function, with no component and
no wiring.

## Where to go next

The three legs each have a page. [Implicit arguments](./implicit-arguments.md) is the value leg, where a
context field arrives as a function parameter. [Abstract types](./abstract-types.md) is the type leg,
and the one that changes how a codebase's signatures age.

[Consumer and provider traits](./consumer-and-provider-traits.md) is the other half of what a component
is: this page covers how an implementation states what it needs, and that one covers how a call reaches
the implementation at all. [Checking your wiring](./check-traits.md) takes up the cost above — why the
check is late, and what to do about it.

For the constructs themselves, [`#[uses]`](/docs/reference/attributes/uses) declares a capability,
[`#[implicit]`](/docs/reference/attributes/implicit) a value, and
[`#[use_type]`](/docs/reference/attributes/use_type) a type;
[`#[blanket_trait]`](/docs/reference/macros/blanket_trait) generates the plain-Rust blanket impl this
page starts from.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
