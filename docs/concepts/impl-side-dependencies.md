---
sidebar_label: 'Impl-side dependencies'
sidebar_position: 3
---

# Impl-side dependencies

A trait exposes operations that callers can use, while an implementation may need additional
traits, values, or types to carry them out. A requirement kept on the implementation without
appearing in the trait's interface is an **impl-side dependency**. Callers can then depend on the
trait alone, and the compiler checks the implementation's requirements when resolving that trait
for a context.

This page develops that separation in ordinary Rust blanket implementations, then applies it to
CGP providers. It covers dependencies on traits, field values, and context-selected types,
including why types used in public signatures must remain available through the interface.

## What a `where` clause costs the callers above it

For example, a generic greeting function can obtain a name through a `HasName` bound on the context:

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

Calling this function from generic code means establishing `HasName` there too. If another bound
does not already imply it, the caller can repeat the requirement:

```rust
pub fn greet_twice<Context>(context: &Context) -> String
where
    Context: HasName,
{
    format!("{} {}", greet(context), greet(context))
}
```

The calls to `greet` are why `greet_twice` needs `HasName`, even though it never reads the name
itself. Further generic callers must supply or imply the same bound. If `greet` gains another
dependency, those forwarding layers may need new bounds as well. Repeating them is straightforward
in a small API, but maintaining them becomes harder as the chain grows.

## Moving the requirement onto the implementation

A trait gives the caller a way to request the greeting operation itself. Its
[blanket implementation](/docs/reference/glossary#blanket-implementation) can supply that operation
for contexts that provide a name:

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

Here, the `HasName` bound belongs to the implementation. Since `CanGreet` does not require it as a
[supertrait](/docs/reference/glossary#supertrait), the caller can ask for `CanGreet` alone:

```rust
pub fn greet_twice<Context>(context: &Context) -> String
where
    Context: CanGreet,
{
    format!("{} {}", context.greet(), context.greet())
}
```

When a concrete type uses this blanket implementation, the compiler still checks `HasName`.
That check makes `CanGreet` available for the type, but the implication only runs one way: a caller
bounded by `CanGreet` cannot use `name()` through that bound. If it needs the name too, it must
establish `HasName` as well.

Keeping the requirement on the implementation also limits how far a change must reach. If the
greeting later needs another dependency, its implementation can add a bound while `greet_twice`
keeps `Context: CanGreet`. The concrete contexts must satisfy the new bound, so some may stop
compiling even though the generic caller's signature stays the same.

This separation already works in ordinary Rust. CGP extends it with separate provider types and
wiring when several implementations need to coexist, including ones with overlapping context
bounds. [Consumer and provider traits](./consumer-and-provider-traits.md) explains that extension.

## Two providers, two sets of requirements, one interface

For email sending, the useful requirements depend on how the message is handled. An SMTP provider
needs a server address, while a recording provider needs a buffer. Each can request its own getter
without adding that requirement to the shared interface:

```rust
use cgp::prelude::*;

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

Each `#[uses]` attribute adds a context bound to its provider's implementation. The bound
`HasSmtpServer` supplies `smtp_server() -> &str`, while `HasSentEmails` supplies
`sent_emails() -> &RefCell<Vec<String>>`. The SMTP body is omitted here, and the recording body
shows how it stores a message in the buffer.

Those getter requirements stay with the providers, so a caller can use either behavior through
`CanSendEmail` alone:

```rust
pub fn notify<Context>(context: &Context)
where
    Context: CanSendEmail,
{
    context.send_email("a@b.c", "hi");
}
```

A production `App` can select `SendViaSmtp`, while a `TestApp` selects `RecordEmails`. These contexts
represent applications, and each supplies the dependencies of its chosen provider. That lets
`notify` accept both with the same bound. Since the test context selects the recording provider,
it only needs the buffer and has no need for an SMTP server.

## Traits, values, and types {#the-three-things-an-implementation-can-ask-for}

The email providers ask the context for behavior through getter traits. Other implementations
may need a field value or a type choice. CGP expresses these dependencies with attributes that
generate ordinary Rust bounds:

- **Traits:** `#[uses(Trait)]` requires the context to implement a trait. It accepts ordinary
  Rust traits such as `AsRef<[u8]>` as well as CGP consumer traits.
- **Values:** An `#[implicit]` argument reads a field from the context and generates the field-access
  requirement on the implementation.
- **Types:** `#[use_type(Trait.Type)]` names an associated type chosen by the context and adds the
  bound needed to use it. A type used in a public signature also needs to be available through
  the interface.

For the greeting example, an implicit argument can request the name value directly from the
context while keeping it out of the public method signature:

```rust
#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_impl(new GreetByName)]
impl Greeter {
    fn greet(&self, #[implicit] name: &str) -> String {
        format!("Hello, {name}!")
    }
}
```

The macro turns `name` into a field-access requirement on `GreetByName`'s implementation and a
local binding in its body. It removes the argument from the public signature, so callers still
use `greet()` without supplying a name. The compiler checks that the context supports the access;
the value is read when the method runs. [Implicit arguments](./implicit-arguments.md) explains
how the field name, type, and borrowing determine that requirement.

## Keeping type choices on the context {#type-dependencies-and-why-they-need-no-parameter}

A context can supply types as well as values. To see how that changes an interface, consider a
job runner that accepts a runtime and a storage backend, with a generic error type chosen by its
caller. With the execution body omitted, its signature looks like this:

```rust
pub fn run_job<E, R, S>(store: &S, runtime: &R) -> Result<(), E>
where
    E: From<S::Error> + From<R::Error>,
    R: Runtime,
    S: Store,
{
    /* job execution omitted */
}
```

A generic forwarding function may need to carry `E`, `R`, and `S` and establish their bounds even
if it only passes the values to `run_job`. Concrete callers can often infer or fix these types,
so the declarations do not necessarily spread to every caller. The maintenance work arises in
generic layers that must preserve choices they do not otherwise use.

Putting the error choice on the context changes what those layers need to name. The associated
type `HasErrorType::Error` lets a component describe its result without a separate `E` parameter:

```rust
#[cgp_component(JobRunner)]
#[use_type(HasErrorType.Error)]
pub trait CanRunJob {
    fn run_job(&self) -> Result<(), Error>;
}
```

Here, `#[use_type]` imports `Error` into the signature and adds `HasErrorType` as a supertrait of
`CanRunJob`. Because callers receive values of that type, this dependency remains part of the
interface. Runtime and storage requirements can stay on the job provider as long as its public
methods do not expose their types.

An operation that forwards to `run_job` can share that associated error:

```rust
#[cgp_fn]
#[uses(CanRunJob)]
#[use_type(HasErrorType.Error)]
pub fn run_jobs(&self) -> Result<(), Error> {
    self.run_job()?;
    self.run_job()
}
```

The generated `RunJobs` trait exposes this forwarding operation. Its blanket implementation
requires `CanRunJob` to make the calls, while `#[use_type]` adds `HasErrorType` to both the trait
and the implementation so they can name the result. Code above this layer can use `RunJobs` alone:

```rust
pub fn run_batch<Context>(context: &Context) -> Result<(), Context::Error>
where
    Context: RunJobs,
{
    context.run_jobs()
}
```

The `RunJobs` bound implies `HasErrorType`, so `run_batch` can name `Context::Error` without another
bound or a separate error parameter. It needs neither `CanRunJob` nor the job provider's runtime
and storage requirements in its signature. Those requirements still have to be satisfied by the
concrete context and its selected provider before the caller can use it.

The shared error also means one error choice per context. If a caller needs a different result
error, it can convert at the boundary or use another context. By comparison, the earlier generic
`E` parameter leaves that choice open at each call.

Associated types reduce the separate parameters that callers carry, but types used in public
arguments and results must still be available through the interface. A type used only inside a
provider can remain an impl-side dependency. [Abstract types](./abstract-types.md) develops
that distinction.

## What it costs

The caller's smaller interface leaves more for the context author to establish. Assembling a
context means following its wiring to the chosen provider and inspecting the dependencies there,
because the trait alone does not describe everything that implementation needs.

Recording the wiring also leaves its requirements to be checked when the trait is used or
explicitly checked. A missing trait or field can remain undetected until then. If a provider gains
a new bound, an existing context may stop compiling even though the consumer trait stays the same.

The resulting errors can describe generated bounds rather than the dependencies as you wrote them.
For example, an implicit field may appear as a missing `HasField` implementation with a nested
type-level field name. [`check_components!`](/docs/reference/macros/check_components) puts the check
where you choose, usually beside the wiring, and helps expose the missing dependency. For the
classes it recognizes, [`cargo cgp check`](/docs/cargo-cgp/check) leads with the root cause, though
it does not yet reshape every class. [Checking your wiring](./check-traits.md) explains those
diagnostics and their limits.

When one blanket implementation gives you enough reuse, ordinary Rust can express it directly,
or [`#[cgp_fn]`](/docs/reference/macros/cgp_fn) can generate it from a function. Components and
wiring are useful when you need alternative providers. A plain generic function is also a good
fit when its explicit parameters and bounds are the interface you want callers to see.

## Where to go next

To apply this separation, consider which requirements a caller needs to use directly and which
belong to the selected implementation. These pages explain the dependency forms and how to
check them:

- [Implicit arguments](./implicit-arguments.md): Reading context values without public method arguments.
- [Abstract types](./abstract-types.md): Sharing context-selected types and deciding which reach the interface.
- [Consumer and provider traits](./consumer-and-provider-traits.md): Connecting a caller to its implementation.
- [Checking your wiring](./check-traits.md): Verifying the selected provider's requirements.
- [`#[uses]`](/docs/reference/attributes/uses),
  [`#[implicit]`](/docs/reference/attributes/implicit), and
  [`#[use_type]`](/docs/reference/attributes/use_type): The dependency syntax used here.
- [`#[cgp_fn]`](/docs/reference/macros/cgp_fn) and
  [`#[blanket_trait]`](/docs/reference/macros/blanket_trait): Generating blanket implementations.
- [Comparison: Dependency injection](/docs/comparisons/dependency-injection): How impl-side dependencies compare with constructor parameters and containers.
- [Comparison: Capabilities](/docs/comparisons/capabilities): How context requirements relate to the different meanings of "capability".

---

*An AI agent wrote this page using the CGP knowledge base. Its content was verified against the
library's source. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
