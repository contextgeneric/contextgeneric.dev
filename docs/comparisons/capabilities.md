---
sidebar_label: 'Capabilities'
sidebar_position: 11
description: 'Distinguish capability models and compare their guarantees with CGP dependency checks and context fields.'
---

# Capabilities

CGP can declare dependencies and carry capability values, but it does not enforce a security
boundary. It is a language extension for Rust, with pluggable trait implementations at
compile-time, implemented as a library on stable Rust whose consumer traits are ordinary Rust
traits; the [Introduction](/docs/) covers the basics. This page separates object capabilities,
effects as capabilities, and other uses of the term, then explains which properties CGP shares
and which must come from another system.

## In your terms

A **context** is the type a CGP method runs on, supplying data through fields and implementations
through wiring. In these examples it represents an application. A context field can hold an
object-capability value, while a provider's declared dependencies describe what it needs from
that context.

| In capability-oriented code | In CGP |
| --- | --- |
| A requirement on a computation's environment | An **impl-side dependency**, declared with `#[uses]` or `#[implicit]` |
| An environment supplying a required value | A context value with the required field |
| A capability value passed to an operation | A context field borrowed by a provider |
| Checking that requirements are supplied | `check_components!` verifies declared dependencies |
| An unforgeable authority-bearing reference | Must be supplied by the field's value type; a trait bound alone does not provide it |
| Excluding ambient authority | Not enforced by CGP; ordinary Rust APIs and globals remain accessible |

## The idea, briefly

The word capability describes several different mechanisms. Object capabilities concern authority
over resources; effect capabilities concern operations available in a scope; other uses concern
privileges, ownership, or implicit dependencies. Their different guarantees determine how each
relates to CGP.

### The object-capability model

An object capability combines a reference to a resource with authority to use it. Code obtains
access through references it is given or can derive from existing authority. This makes the
reference graph relevant to the principle of least authority: give each component only the access
it needs. [Miller's account](https://papers.agoric.com/assets/pdf/papers/robust-composition.pdf)
and [Capability Myths Demolished](https://papers.agoric.com/assets/pdf/papers/capability-myths-demolished.pdf)
explain the model and its security properties.

A capability-safe environment excludes ambient authority that would bypass those references.
An unrestricted global file-opening function would violate that rule. Passing a reference delegates
authority; a wrapper can expose fewer operations or mediate access so it can later revoke it.
These are properties of the supplied objects and their environment, rather than of a dependency
annotation alone.

Pony gives its main actor an environment from which it obtains authority. The following fragment
uses `env.root` to create file access, following the
[Pony tutorial](https://tutorial.ponylang.io/object-capabilities/object-capabilities.html):

```pony
use "files"

actor Main
  new create(env: Env) =>
    // `env.root` is the only source of authority; it is passed in, never global.
    let auth: FileAuth = FileAuth(env.root)
    let path = FilePath(auth, "/var/app/config.txt")
    match OpenFile(path)
    | let file: File => env.out.print(file.read_string(100))
    else env.out.print("could not open")
    end
```

### Capability-based security in systems

Capability systems can enforce authority below the language level. Examples include kernel-object
handles in seL4, hardware support in [CHERI](https://www.cl.cam.ac.uk/research/security/ctsrd/cheri/),
and the handles supplied through [WASI](https://github.com/WebAssembly/WASI/blob/main/docs/DesignPrinciples.md).
Their enforcement mechanisms differ, so a library that accepts handles should not be assumed to
provide the same isolation as an operating system or sandbox.

Rust's `cap-std` exposes resource access through explicit handles. Its `Dir` API limits file
operations to a directory, while opening a directory through ambient authority requires an explicit
marker. This fragment illustrates the distinction:

```rust
use cap_std::ambient_authority;
use cap_std::fs::Dir;

fn read_config(dir: &Dir) -> std::io::Result<String> {
    // File access through `dir` stays within the directory it represents.
    let mut text = String::new();
    std::io::Read::read_to_string(&mut dir.open("config.txt")?, &mut text)?;
    Ok(text)
}

// Ambient authority is exercised once, and the call marks it.
let dir = Dir::open_ambient_dir("/var/app", ambient_authority())?;
read_config(&dir)?;
dir.open("../secret.txt")   // Err: a path led outside of the filesystem
```

The confinement applies to operations performed through `dir`. Arbitrary Rust code can still call
`std::fs` directly if its execution environment permits it. The
[cap-std README](https://github.com/bytecodealliance/cap-std) explains both the handle-based APIs and
the limit of a library-level discipline.

### Privilege bits and reference qualifiers

Linux capabilities divide superuser privileges into units such as `CAP_NET_BIND_SERVICE`.
They are per-thread privilege attributes, rather than references to individual resources. The
[capabilities manual](https://man7.org/linux/man-pages/man7/capabilities.7.html) describes that model.

Pony's reference capabilities describe aliasing and access to values. Qualifiers such as `iso`,
`val`, and `ref` support safe sharing and data-race prevention. Rust ownership and borrowing address
related concerns through different rules. The
[Pony reference-capability tutorial](https://tutorial.ponylang.io/reference-capabilities/)
explains these qualifiers. Neither these qualifiers nor Linux privilege bits are mechanisms CGP adds.

### Effects as capabilities

Effect-capability systems make an operation depend on access to a handler-provided capability.
Effekt expresses the capabilities a computation requires through its effect types and controls
how they can escape their scope. The paper
[Effects as Capabilities](https://dl.acm.org/doi/10.1145/3428194) develops this approach.

Scala's capture checking tracks references to capabilities in types, while its safer-exceptions
feature uses `CanThrow` to express permission to throw a checked exception. The following example
uses the experimental safer-exceptions feature in Scala 3.8.4:

```scala
import language.experimental.saferExceptions

class LimitExceeded extends Exception
val limit = 10e9

def f(x: Double): Double throws LimitExceeded =
  if x < limit then x * x else throw LimitExceeded()

try println(List(1.0, 2.0, 3.0).map(f).sum)   // the try supplies the CanThrow capability
catch case ex: LimitExceeded => println("too large")
```

The surrounding `try` supplies the `CanThrow` evidence needed by `f`. This illustrates static
permission to perform an operation, rather than a runtime operating-system privilege. See Scala's
[CanThrow documentation](https://docs.scala-lang.org/scala3/reference/experimental/canthrow.html)
and [capture-checking reference](https://docs.scala-lang.org/scala3/reference/experimental/cc.html).
CGP's declared requirements resemble part of this structure, but CGP does not supply capture checking.

### Capabilities in Rust discussions

Rust discussions use capability terminology for implicit dependencies, sandboxed resources, and
ownership tokens. The intended guarantee matters more than the shared word:

- **Implicit dependencies:** the contexts-and-capabilities proposal supplies values such as an
  arena through a context, reducing parameter forwarding. See
  [Mandry's proposal](https://tmandry.gitlab.io/blog/posts/2021-12-21-context-capabilities/).
- **Sandboxed resources:** `cap-std` and WASI express access through handles, with different
  enforcement boundaries.
- **Ownership tokens:** a value with a controlled constructor represents access to a resource,
  such as a peripheral. The [Embedded Rust Book](https://docs.rust-embedded.org/book/peripherals/singletons.html)
  explains the singleton pattern, and [PermRust](https://arxiv.org/abs/2506.11701) explores token-based
  permissions.

A request for capabilities in Rust can therefore mean either a more convenient API or a stronger
restriction on what code may do. CGP addresses dependency declaration and selection; its wiring
alone does not impose the latter restriction.

### The properties that distinguish the models

The following properties help identify which mechanism a program needs:

| Mechanism | What it represents | Where its guarantee comes from |
| --- | --- | --- |
| Object capability | Authority carried by a reference | Controlled references and an environment that excludes bypasses |
| Effect capability | Access to an operation or handler | The language's effect, scope, or capture rules |
| Rust contexts proposal | An implicitly supplied dependency | The proposed context-binding and type-checking rules |
| Ownership token | Access represented by a value | Controlled construction and ownership or borrowing |
| CGP dependency | A trait or field an implementation requires | Rust trait checking against the selected context |

## How CGP expresses it

CGP declares what a provider needs from its context and checks that the context supplies it.
Those requirements may include authority-bearing values, but CGP does not make every dependency
an authority token. It also does not restrict access to globals or check the escape of capability
references beyond Rust's ordinary type and lifetime rules.

### Providers declare requirements on a context

A provider uses `#[uses]` for trait dependencies and `#[implicit]` for field dependencies.
This fragment assumes a `CanGreet` component whose method returns a `String`:

```rust
#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self, #[implicit] name: &str) -> String {
        format!("Hello, {name}!")
    }
}

#[derive(HasField)]
pub struct App {
    pub name: String,
}

delegate_components! {
    App {
        GreeterComponent: GreetHello,
    }
}

check_components! { App { GreeterComponent } }
```

`GreetHello` needs a `name` field, and `App` supplies it. `check_components!` verifies the field
requirement for the selected provider; constructing `App { name }` supplies its runtime value.
`App` is an [environmental context](/docs/reference/glossary#environmental-context) representing an application, and the component is [self-targeted](/docs/reference/glossary#self-targeted-component).
This is a dependency relationship: the `String` does not confer protected authority.

Contexts can also determine [abstract types](/docs/concepts/abstract-types), such as the error type
shared by their providers. Selecting such a type is separate from supplying an authority-bearing
value of that type. A type choice alone does not grant access to a resource.

### An object capability can live in the context

A context can carry a `cap-std` directory handle to a provider that declares it as a dependency.
This example reads configuration using that handle:

```rust
use cap_std::fs::Dir;

#[cgp_component(ConfigReader)]
pub trait CanReadConfig {
    fn read_config(&self) -> std::io::Result<String>;
}

#[cgp_impl(new ReadConfigFromDir)]
impl ConfigReader {
    fn read_config(&self, #[implicit] config_dir: &Dir) -> std::io::Result<String> {
        let mut text = String::new();
        std::io::Read::read_to_string(&mut config_dir.open("config.txt")?, &mut text)?;
        Ok(text)
    }
}

#[derive(HasField)]
pub struct App {
    pub config_dir: Dir,
}

delegate_components! {
    App {
        ConfigReaderComponent: ReadConfigFromDir,
    }
}

check_components! { App { ConfigReaderComponent } }
```

`cap-std` constrains file access performed through `config_dir`, while CGP verifies that `App`
supplies the required field. `App` is again an environmental context with a self-targeted component.
The mechanisms compose: the value type supplies resource-access behavior, and CGP supplies the
dependency declaration and static wiring check. An allocator or arena can be passed through a context
field in the same way, as discussed in [Rust's own proposals](./rust-language-proposals.md).

### CGP does not remove ambient authority

A provider can access ordinary Rust APIs beyond its context dependencies. `ReadConfigFromDir`
could call `std::fs::File::open(...)` directly if the execution environment allows it, and CGP
would not reject the call. Its declared bounds describe requirements on the context, not a complete
list of effects or authority exercised by the body.

### A trait bound does not create an authority token

A `HasField` bound establishes access to a field of a particular name and type. It does not
establish who may create the field's value. When the field contains a capability, the relevant
construction and access restrictions come from that value's type and execution environment.
Rust types with private constructors can implement controlled tokens; CGP can carry those tokens
without changing their guarantees.

### Wiring is static; capability values can vary at runtime

A concrete context type fixes its provider selections and field types. Its values can still hold
different handles: two `App` instances can refer to different directories, including a temporary
directory used by a test. Their filesystem authority therefore differs even though their wiring
is identical.

Delegation, attenuation, and revocation must come from the capability implementation. A context
can hold a wrapper that limits access or checks whether access has been revoked. A
[higher-order provider](/docs/concepts/higher-order-providers) can also wrap behavior, but wrapping
alone is not proof of attenuation if other access paths remain available. CGP does not add these
security properties or prevent runtime capability values from implementing them.

## What each approach costs

Object capabilities support reasoning about authority through references, but that reasoning needs
an enforcement boundary. Retrofitting a handle-based API into an unrestricted language does not
remove other APIs that exercise ambient authority. Applications must account for those bypasses
or use an environment that excludes them. The
[cap-std documentation](https://github.com/bytecodealliance/cap-std) makes this library-level limit
explicit.

Effect-capability systems add scope and escape rules that programmers must understand.
Effekt's second-class treatment restricts where capabilities can be stored or returned, while
Scala's capture checking tracks their use through types. These rules support guarantees that a
plain dependency declaration does not provide. The
[Effekt paper](https://dl.acm.org/doi/10.1145/3428194) and
[Scala reference](https://docs.scala-lang.org/scala3/reference/experimental/cc.html) explain the
respective designs.

CGP adds declarations, wiring, and compile-time work without supplying confinement or capture
checking. Programs that need those properties must obtain them elsewhere. Its generated trait
machinery also affects diagnostics: [`cargo cgp check`](/docs/cargo-cgp/check) leads with the root
cause for the classes it recognizes, and the tool is a v0.1.0-alpha that does not yet reshape every
class. The [Modularity Hierarchy](/docs/concepts/modularity-hierarchy) compares this machinery
with simpler Rust abstractions.

## Where a capability system is the better choice

Use an enforcing capability system when the requirement is to limit what code may access.
A capability-safe language or sandbox can exclude ambient access paths. Within Rust, `cap-std`
helps express handle-based access, while an appropriate sandbox is needed to constrain code that
could otherwise bypass those handles. Ownership tokens address controlled access to resources
through Rust's construction and borrowing rules.

CGP fits the separate requirement of reusable implementations with declared dependencies and
choices per context. It can carry capability values from those systems, but a wiring check is not
a security audit or proof of confinement.

## What to expect that differs

CGP checks declared context requirements rather than all authority used by a body.
Ordinary Rust globals and APIs remain available unless another mechanism restricts them.

A field dependency identifies a value the implementation needs. Whether that value grants
protected authority depends on its type and how it was obtained. A `String` and a directory handle
can both be dependencies without having the same security meaning.

The context type fixes wiring, while runtime values determine which resources its handles reach.
Any delegation or revocation behavior belongs to those handles and wrappers, not to the wiring
macro.

CGP does not add an escape checker. Rust's ownership, borrowing, and lifetime rules still apply,
and a value can be cloned or moved only when its type and the surrounding code permit it.
CGP adds neither Effekt's second-class restriction nor Scala's capture sets.

CGP terminology keeps these distinctions visible. A component defines a trait, a provider requires
traits and reads fields, and a context supplies those requirements. The term capability is reserved
for the external mechanisms and values whose properties justify it.

## Where to go next

These pages explain the dependencies and related language proposals:

- [Impl-side dependencies](/docs/concepts/impl-side-dependencies): requirements declared by providers.
- [Implicit arguments](/docs/concepts/implicit-arguments): reading values from context fields.
- [Algebraic effects](./algebraic-effects.md): operations, handlers, and the continuation boundary.
- [Rust's own proposals](./rust-language-proposals.md): the contexts-and-capabilities proposal.

## Sources

The `cap-std` snippets were compiled with `cap-std` 3 and the CGP snippet against `cgp` `0.8.0-alpha`
with a `check_components!` assertion per wired context; the Scala snippet was compiled with Scala
3.8.4. The Pony snippet was checked against the standard library's documented signatures for
`FileAuth`, `FilePath`, and `OpenFile` rather than compiled.

- [Dennis & Van Horn, *Programming Semantics for Multiprogrammed Computations* (CACM 1966)](https://dl.acm.org/doi/10.1145/365230.365252): the origin of the capability.
- [Miller, *Robust Composition* (PhD thesis, 2006)](https://papers.agoric.com/assets/pdf/papers/robust-composition.pdf) and [Miller, Yee & Shapiro, *Capability Myths Demolished* (2003)](https://papers.agoric.com/assets/pdf/papers/capability-myths-demolished.pdf): the object-capability model, its defining properties, delegation, attenuation, revocation, and the myths.
- [Wikipedia, *Object-capability model*](https://en.wikipedia.org/wiki/Object-capability_model) and [*Capability-based security*](https://en.wikipedia.org/wiki/Capability-based_security): the definitions and the systems implementing the model.
- [Pony tutorial, *Object Capabilities*](https://tutorial.ponylang.io/object-capabilities/object-capabilities.html) and [*Reference Capabilities*](https://tutorial.ponylang.io/reference-capabilities/): the unforgeable-token definition, `env.root`, and the aliasing qualifiers that share the word.
- [Austral specification](https://austral-lang.org/spec/spec.html), [CHERI](https://www.cl.cam.ac.uk/research/security/ctsrd/cheri/), and [WASI design principles](https://github.com/WebAssembly/WASI/blob/main/docs/DesignPrinciples.md): linear capability values, architectural capabilities, and unforgeable handles without ambient authority.
- [Gohman, *Introducing cap-std*](https://blog.sunfishcode.online/introducing-cap-std/) and the [cap-std README](https://github.com/bytecodealliance/cap-std): capability-oriented APIs, `ambient_authority()`, and the limit that a library cannot sandbox arbitrary Rust code.
- [capabilities(7)](https://man7.org/linux/man-pages/man7/capabilities.7.html): Linux capabilities as privilege units.
- [Brachthäuser, Schuster & Ostermann, *Effects as Capabilities* (OOPSLA 2020)](https://dl.acm.org/doi/10.1145/3428194), [Odersky et al., *Scoped Capabilities for Polymorphic Effects* (2022)](https://arxiv.org/abs/2207.03402), [Scala 3 Reference, *Capture Checking*](https://docs.scala-lang.org/scala3/reference/experimental/cc.html), and [*CanThrow Capabilities*](https://docs.scala-lang.org/scala3/reference/experimental/canthrow.html): effect types as capabilities a computation requires, capture sets, and the erased `CanThrow`.
- [Mandry, *Contexts and capabilities in Rust*](https://tmandry.gitlab.io/blog/posts/2021-12-21-context-capabilities/) and [Wuyts, *Nesting Allocators*](https://blog.yoshuawuyts.com/nesting-allocators): the implicit-value sense and the global allocator as ambient authority.
- [Embedded Rust Book, *Peripherals as singletons*](https://docs.rust-embedded.org/book/peripherals/singletons.html) and [Gehring, Rehms & Tschorsch, *PermRust*](https://arxiv.org/abs/2506.11701): ownership tokens as static capabilities.

---

*An AI agent wrote this page using the CGP knowledge base. Its CGP code was verified against the
library's source and its other snippets against the toolchains named in Sources. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
