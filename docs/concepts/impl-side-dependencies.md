---
sidebar_label: 'Impl-side dependencies'
sidebar_position: 3
---

# Impl-side dependencies

An **impl-side dependency** is a requirement stated on an implementation without being part of its
trait interface. Callers can require the capability they use while the compiler checks the selected
implementation's dependencies. This page starts with ordinary Rust blanket implementations, then
shows how CGP providers require capabilities, field values, and context-selected types.

## What a `where` clause costs the callers above it

A generic function exposes its requirements through its signature. This greeting function needs
`HasName` to obtain the name it formats:

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

A generic caller must establish that requirement too. If it has nothing else that implies
`HasName`, it can add the same bound:

```rust
pub fn greet_twice<Context>(context: &Context) -> String
where
    Context: HasName,
{
    format!("{} {}", greet(context), greet(context))
}
```

`greet_twice` does not read the name itself, but its calls to `greet` require the bound.
Further generic callers must also supply or imply it. Adding a dependency to `greet` can therefore
require changes to callers that only forward the call. This is straightforward for a small API;
it becomes harder to maintain when many layers repeat implementation-specific requirements.

## Moving the requirement onto the implementation

A blanket implementation lets callers depend on a capability without naming the dependencies used
to implement it. The trait declares the operation, and the implementation states when it is available:

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

`CanGreet` promises a greeting method without requiring `HasName` as a supertrait. The blanket
implementation supplies that method for every context implementing `HasName`. A generic caller
can require only the greeting capability:

```rust
pub fn greet_twice<Context>(context: &Context) -> String
where
    Context: CanGreet,
{
    format!("{} {}", context.greet(), context.greet())
}
```

The compiler still checks `HasName` when it uses that blanket implementation to establish
`Context: CanGreet` for a concrete type. The requirement remains necessary for the implementation;
it is absent from the caller's trait contract. That separation is the impl-side dependency.

Keeping the interface stable can protect generic callers from implementation changes. If greeting
later needs another dependency, the blanket implementation can add a bound while `greet_twice`
continues to require only `CanGreet`. Concrete contexts must still satisfy the new requirement,
so this does not make every dependency change backward-compatible.

Ordinary Rust already supports this technique. CGP adds separate provider types and wiring when
several implementations need to coexist, including implementations whose context requirements overlap.
[Consumer and provider traits](./consumer-and-provider-traits.md) explains that extension.

## Two providers, two sets of requirements, one interface

Each CGP provider can require different context capabilities while implementing the same interface.
An email sender might need an SMTP server getter, while a test implementation needs access to a
recording buffer:

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

`#[uses(HasSmtpServer)]` adds a context bound to `SendViaSmtp`'s implementation.
`#[uses(HasSentEmails)]` adds a different bound to `RecordEmails`. Here `HasSmtpServer` supplies
`smtp_server()`, and `HasSentEmails` supplies `sent_emails()` as a reference to a mutable recording
buffer. The SMTP body is omitted; the recording body stores each message in that buffer.

`CanSendEmail` exposes neither dependency. A caller needs only that trait, regardless of which
provider supplies it:

```rust
pub fn notify<Context>(context: &Context)
where
    Context: CanSendEmail,
{
    context.send_email("a@b.c", "hi");
}
```

A production `App` can select `SendViaSmtp`, while a `TestApp` selects `RecordEmails`. These contexts
represent applications and supply their chosen provider's dependencies. `notify` accepts either
context through the same bound; the test context does not need an SMTP server merely because
another provider uses one.

## The three things an implementation can ask for

Provider dependencies commonly describe capabilities, values, or types. CGP expresses them with
attributes that generate the corresponding Rust bounds:

- **Capabilities:** `#[uses(Trait)]` requires the context to implement a trait. It accepts ordinary
  Rust traits such as `AsRef<[u8]>` as well as CGP consumer traits.
- **Values:** An `#[implicit]` argument reads a field from the context and generates the field-access
  requirement on the implementation.
- **Types:** `#[use_type(Trait.Type)]` names an associated type chosen by the context and adds the
  bound needed to use it. Whether that bound belongs in the interface depends on where the type appears.

An implicit argument lets the implementation ask for a value without adding a public method argument:

```rust
#[cgp_impl(new GreetByName)]
impl Greeter {
    fn greet(&self, #[implicit] name: &str) -> String {
        format!("Hello, {name}!")
    }
}
```

`GreetByName` reads `name` through the context's field-access implementation. The macro removes the
implicit argument from the method signature, so callers still use `greet()` without passing a name.
[Implicit arguments](./implicit-arguments.md) explains how field names, types, and borrowing determine
that generated requirement.

## Type dependencies, and why they need no parameter

Associated types let a context collect type choices that would otherwise be separate generic
parameters. A job runner, for example, might leave its error type, runtime, and storage backend
open. A function interface can expose those choices directly; with its body omitted, it looks like this:

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

A generic forwarding function may need to carry `E`, `R`, and `S` and establish the same bounds.
Concrete callers can often infer or fix these types, so the declarations do not necessarily spread
to every caller. The maintenance problem arises where intermediate generic layers must preserve
choices they do not otherwise use.

A context-based interface can expose the operation while letting the context determine its error
type. `HasErrorType` supplies an associated `Error`, which the component imports into its signature:

```rust
#[cgp_component(JobRunner)]
#[use_type(HasErrorType.Error)]
pub trait CanRunJob {
    fn run_job(&self) -> Result<(), Error>;
}
```

`CanRunJob` now has a `HasErrorType` supertrait because its return type names `Error`.
This type dependency is part of the interface, not hidden on the implementation. The runtime and
storage requirements can remain on the job provider if its public methods do not expose their types.

A forwarding capability can use that same associated error without introducing a separate `E`
parameter:

```rust
#[cgp_fn]
#[uses(CanRunJob)]
#[use_type(HasErrorType.Error)]
pub fn run_jobs(&self) -> Result<(), Error> {
    self.run_job()?;
    self.run_job()
}
```

`run_jobs` still declares the requirements it uses: `CanRunJob` for calling the job and `HasErrorType`
for naming its result. The `#[use_type]` attribute lets it write `Error` instead of a qualified
associated-type projection. The context determines the concrete error type shared by both operations.

A type used only inside a provider can remain an impl-side dependency. A type appearing in a public
argument or result must be available through the interface. Associated types reduce separate type
parameters and tie related choices to the context; they do not eliminate all type dependencies from
signatures. [Abstract types](./abstract-types.md) explains that choice and its limits.

## What it costs

The trait interface does not tell a context author everything a selected provider needs.
To assemble a context, follow its wiring to the provider and inspect that provider's requirements.
The smaller caller contract comes with more work when configuring an implementation.

Wiring alone does not verify that the context satisfies the provider's dependencies. A missing
capability or field can remain undetected until a check or use requires it. Changing a provider's
bounds can also make an existing context stop compiling even if the consumer trait is unchanged.

Errors may refer to generated bounds rather than the requirement as written in the provider.
A missing implicit field, for example, can appear as an unsatisfied `HasField` bound with a
nested type-level field name. [`check_components!`](/docs/reference/macros/check_components)
checks selected components where you place it, usually beside the wiring, and helps expose the
missing dependency. [`cargo cgp check`](https://github.com/contextgeneric/cargo-cgp) reports readable
causes for recognized cases; its `v0.1.0-alpha` release covers core wiring errors rather than every
class. [Checking your wiring](./check-traits.md) shows the diagnostics and their limits.

A component and wiring are unnecessary when one blanket implementation provides all the reuse you
need. Write that implementation in ordinary Rust, or use
[`#[cgp_fn]`](/docs/reference/macros/cgp_fn) to generate a capability from a function. A plain generic
function remains suitable when its explicit parameters and bounds are the interface callers should see.

## Where to go next

These pages develop the dependency forms and explain how providers are selected and checked:

- [Implicit arguments](./implicit-arguments.md): Reading context values without public method arguments.
- [Abstract types](./abstract-types.md): Sharing context-selected types and deciding which reach the interface.
- [Consumer and provider traits](./consumer-and-provider-traits.md): Connecting a caller to its implementation.
- [Checking your wiring](./check-traits.md): Verifying the selected provider's requirements.
- [`#[uses]`](/docs/reference/attributes/uses),
  [`#[implicit]`](/docs/reference/attributes/implicit), and
  [`#[use_type]`](/docs/reference/attributes/use_type): The dependency syntax used here.
- [`#[cgp_fn]`](/docs/reference/macros/cgp_fn) and
  [`#[blanket_trait]`](/docs/reference/macros/blanket_trait): Generating blanket implementations.

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
