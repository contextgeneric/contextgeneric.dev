---
sidebar_label: 'Capabilities'
sidebar_position: 11
description: 'The five things "capability" names, and which of them CGP resembles: object capabilities, capability-based security, effects as capabilities, and the Rust community''s uses.'
---

# Capabilities

CGP is a language extension for Rust, with pluggable trait implementations at compile-time: a
library on stable Rust in which a trait can have several named implementations and each context
selects one. The [Introduction](/docs/) covers the basics. This page is for the reader who knows
"capabilities" in one of the several senses the word carries: the object-capability model of E and
Pony, capability-based security in seL4, WASI, or `cap-std`, effects as capabilities in Effekt and
Scala, or the Rust community's use of the word for an allocator or runtime a function should receive
without a parameter. CGP is capability-like in the last two senses and is not a capability system in
the first. The page separates the senses, says which requirements CGP meets, and gives a compact way
to say "capability-like, but not a capability system".

## In your terms

A **context** in CGP is the type the method runs on, which supplies the values it needs as its
fields. In most CGP code it is a type you define to stand for an application.

| In a capability system | In CGP |
| --- | --- |
| A requirement a computation places on its context (Effekt, Scala) | An **impl-side dependency**: `#[uses]` for a trait, `#[implicit]` for a value |
| The handler or scope that supplies it | The context, and constructing a value of it |
| A capability value passed to code that needs it | A field of the context, read by the providers that declare it |
| Static discharge of every requirement | `check_components!` |
| An unforgeable reference (E, Pony, WASI) | No counterpart: a CGP bound is a requirement, not a token |
| Removal of ambient authority | No counterpart: Rust keeps `std` and globals reachable |

## The idea, briefly

"Capability" names at least five different things, and a reader who asks for capabilities in Rust
rarely says which. The senses are separated below by the property each insists on.

### The object-capability model

A capability, in the sense Dennis and Van Horn introduced in 1966 and Mark Miller made precise for
programming languages, is "a communicable, unforgeable token of authority" that refers to an object
together with the rights to use it ([Wikipedia, *Capability-based security*](https://en.wikipedia.org/wiki/Capability-based_security)).
The object-capability model identifies that token with an ordinary object reference: to hold the
reference is to hold the authority. Three properties define it. *No designation without authority*:
naming a resource and being allowed to use it are the same act. *No ambient authority*: a request
either carries a capability or fails, so global functions such as `open()` are the counterexample
([Miller, Yee & Shapiro, *Capability Myths Demolished*](https://papers.agoric.com/assets/pdf/papers/capability-myths-demolished.pdf)).
And authority spreads only along existing references: "only connectivity begets connectivity"
([Miller, *Robust Composition*](https://papers.agoric.com/assets/pdf/papers/robust-composition.pdf)),
so reachability in the object graph bounds what any object can do, which is the reasoning the
*principle of least authority* rests on. Delegation is passing the reference, attenuation is wrapping
it, and revocation is closing a gate in the wrapper, all runtime acts on runtime values. The languages
built on the model, E, Pony, Newspeak, Austral, and Hardened JavaScript, remove global mutable state
and global I/O, because a global is ambient authority. Pony's tutorial states the rule: global
variables are "ambient authority", so Pony has none, and a program obtains file access only from the
`env.root` authority its `Main` actor is handed
([Pony tutorial, *Object Capabilities*](https://tutorial.ponylang.io/object-capabilities/object-capabilities.html)):

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

The same model applied below the language gives a process handles to kernel objects and no other way
to name them: KeyKOS, seL4, Fuchsia, and Capsicum, with CHERI moving it into hardware
([CHERI](https://www.cl.cam.ac.uk/research/security/ctsrd/cheri/)). WASI "has no ambient
authorities" and its handles "are unforgeable"
([WASI design principles](https://github.com/WebAssembly/WASI/blob/main/docs/DesignPrinciples.md)).
Rust's `cap-std` brings the discipline to the standard library's surface: its APIs "don't access
files, directories, network addresses, clocks, or other external resources implicitly, but instead
operate on handles that are explicitly passed in", the one exercise of ambient authority is marked by
an explicit `ambient_authority()` argument, and the crate is candid that it "can't prevent arbitrary
Rust code from using `std::fs`'s path-oriented APIs"
([Gohman, *Introducing cap-std*](https://blog.sunfishcode.online/introducing-cap-std/)). Compiled with
`cap-std` 3:

```rust
use cap_std::ambient_authority;
use cap_std::fs::Dir;

fn read_config(dir: &Dir) -> std::io::Result<String> {
    // `dir` is the only authority this function holds; it cannot reach outside it.
    let mut text = String::new();
    std::io::Read::read_to_string(&mut dir.open("config.txt")?, &mut text)?;
    Ok(text)
}

// Ambient authority is exercised once, and the call marks it.
let dir = Dir::open_ambient_dir("/var/app", ambient_authority())?;
read_config(&dir)?;
dir.open("../secret.txt")   // Err: a path led outside of the filesystem
```

### Two senses that share only the word

Linux "capabilities" divide the superuser's privileges into units such as `CAP_NET_BIND_SERVICE`,
held as a per-thread attribute ([capabilities(7)](https://man7.org/linux/man-pages/man7/capabilities.7.html)).
They are privilege bits, not references to resources. Pony's *reference capabilities*, `iso`, `trn`,
`ref`, `val`, `box`, and `tag`, are aliasing qualifiers whose purpose is data-race freedom
([Pony tutorial, *Reference Capabilities*](https://tutorial.ponylang.io/reference-capabilities/)). A
Rust reader already has that second thing as ownership, `&`, and `&mut`. Neither sense enters the CGP
comparison.

### Effects as capabilities

Effekt's designers describe their language as "effects as capabilities": "effect types express which
capabilities a computation requires from its context", given semantics by translation to a calculus in
explicit capability-passing style
([Brachthäuser, Schuster & Ostermann, OOPSLA 2020](https://dl.acm.org/doi/10.1145/3428194)). A
capability here is a value a handler introduces and that must be in scope for an operation to be
performed, and to keep this sound Effekt treats capabilities as second-class: passable down, not
returned or stored. Scala 3's experimental *capture checking* reaches the same place: a capability is
a parameter or local whose type carries a capture set, and the checker prevents it escaping the scope
that introduced it ([Scala 3 Reference, *Capture Checking*](https://docs.scala-lang.org/scala3/reference/experimental/cc.html)).
The concrete application is `CanThrow`, an erased class: a `throw` requires a `CanThrow[E]`, and a
`try` creates one. Compiled with Scala 3.8.4:

```scala
import language.experimental.saferExceptions

class LimitExceeded extends Exception
val limit = 10e9

def f(x: Double): Double throws LimitExceeded =
  if x < limit then x * x else throw LimitExceeded()

try println(List(1.0, 2.0, 3.0).map(f).sum)   // the try supplies the CanThrow capability
catch case ex: LimitExceeded => println("too large")
```

This sense keeps one thing from the object-capability model, that authority to perform an operation is
a value the context supplies, and drops the runtime half. It is a *static* discipline, which is why it
is the sense CGP can be compared with directly.

### The Rust community's word

Rust programmers use the word in three threads. The first is *implicit values*: Tyler Mandry's
contexts-and-capabilities proposal declares a `capability` such as an arena and lets an impl require it
in a `with` clause, so an allocator or logger need not be threaded through every signature
([Mandry](https://tmandry.gitlab.io/blog/posts/2021-12-21-context-capabilities/)), and Yoshua Wuyts
describes the pain: "the global allocator acts, in a capability-sense, as an ambient authority"
([Wuyts, *Nesting Allocators*](https://blog.yoshuawuyts.com/nesting-allocators)). The second is
*sandboxing*, the object-capability sense proper, as in `cap-std` and WASI. The third is *ownership
tokens*: the embedded Rust book makes each peripheral a value obtained once, so that "to call the
`read_speed()` method, we must have ownership or a reference to a `SerialPort` structure"
([Embedded Rust Book](https://docs.rust-embedded.org/book/peripherals/singletons.html)), and PermRust
generalizes the idea into a token-based permission system
([Gehring, Rehms & Tschorsch](https://arxiv.org/abs/2506.11701)). A reader who wants "capabilities in
Rust" may mean any of the three.

### The properties that tell the senses apart

| Property | Object capabilities | Effects as capabilities | Rust contexts proposal | Ownership tokens |
| --- | --- | --- | --- | --- |
| Designation and authority are one reference | yes | partly | no: a named implicit value | yes, for the resource the token stands for |
| No ambient authority, enforced | yes, by removing globals | no, unless the language also removes them | no | no |
| Unforgeable | yes | yes, by private constructors or erasure | not addressed | yes, by a private constructor |
| Provisioned at runtime | yes | value at runtime, scope lexical | value at runtime, binding lexical | value at runtime |
| Delegation, attenuation, revocation | all three, at runtime | delegation and attenuation; no revocation | delegation by scope | delegation by move or borrow |
| Confinement reasoning | reachability in the object graph | capture sets bound escape | not addressed | ownership bounds aliasing |

## How CGP expresses it

CGP shares the effects-as-capabilities reading almost exactly and the object-capability reading only in
part. A provider's [impl-side dependencies](/docs/concepts/impl-side-dependencies) are the requirements
a computation places on its context, the context supplies them, and
[`check_components!`](/docs/reference/macros/check_components) verifies that every requirement is met.
CGP does not remove ambient authority, does not make its requirements unforgeable, and does not
provision them at runtime.

### A requirement on the context is a capability in the effect-system sense

A provider declares what it needs with `#[uses]` for traits and `#[implicit]` for values, and the
consumer trait a caller invokes hides both:

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

Read as an effect type, `GreetHello` requires one thing of its context, a `name` field, and `App`
supplies it. Constructing `App { name }` is the `try` that supplies a `CanThrow`, or the `with` block
that supplies Mandry's arena. The parallel extends to the type level: an
[abstract type](/docs/concepts/abstract-types) such as a context's `Error` is a requirement the
context discharges by wiring, which no capability system expresses. `App` here is an **environmental
context**, a type standing for an application, and `GreetHello` is self-targeted.

### An object capability can live in the context

Where the value a provider requires is itself an object capability, CGP carries it and adds a static
check that it reaches the code that needs it. A `cap-std` `Dir` stored as a context field is an object
capability in the model's sense, and a provider that reads configuration through it holds no other
filesystem authority in its body. This compiles against `cgp` `0.8.0-alpha` and `cap-std` 3:

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

The division of labor is the point. `cap-std` supplies the unforgeable handle and the runtime
confinement; CGP supplies the declaration that `ReadConfigFromDir` needs a `Dir` and the compile-time
check that `App` has one. The same arrangement serves an arena-allocating deserializer, where the
arena is a context field and the provider reads it as an implicit argument, which the
[Rust proposals](./rust-language-proposals.md) page relates to Mandry's example.

### CGP does not remove ambient authority

The property that makes a language capability-safe is that code can reach only what it is handed, and
Rust does not have it. `ReadConfigFromDir` could call `std::fs::File::open("/etc/passwd")` in its
body and CGP would neither notice nor object. The provider's declared requirements bound what it
reaches *through the context*, which is useful for reading and for testing, but they do not bound what
it reaches through `std`, through statics, or through the global allocator. A capability-safe language
forbids the global; CGP is a library in a language that permits it. A page that suggested a CGP
provider is confined to its declared dependencies would be claiming something Rust cannot deliver.

### A requirement is not an unforgeable token

An object capability is unforgeable because the only way to obtain the reference is to be given it. A
CGP requirement is a trait bound on the context, and any context that carries a field of the right
name and type satisfies a `HasField<Symbol!("config_dir")>` bound. The bound says what the provider
needs; it does not certify who may construct the context. Authority, where there is any, lives in the
*value* stored in the field: a `Dir` is unforgeable because `cap-std` made it so, and a `String` named
`name` is not a capability at all. Rust can express unforgeable tokens as types with private
constructors, and CGP can carry such a token as a field or fix one as an abstract type, but the token
pattern is Rust's.

### Provisioning is fixed per context type

In the object-capability model authority is provisioned at runtime and the graph changes while the
program runs. In CGP the set of requirements a context satisfies, and the providers it satisfies them
with, are part of the context *type*, fixed when `delegate_components!` is written. Only the field
*values* are dynamic: two `App` values may carry two different `Dir` handles, and a test may carry a
`Dir` opened on a temporary directory. That is provisioning of values, not of authority structure.
Attenuation means a [higher-order provider](/docs/concepts/higher-order-providers) that wraps an inner
one, decided statically; revocation has no counterpart; and bindings are flat, with no nested scope in
which a provider could be shadowed.

## What each approach costs

Object capabilities are valued for what their reasoning buys: least authority as a property of the
object graph rather than a policy someone maintains, structural immunity to confused-deputy attacks,
and delegation and attenuation as ordinary programming
([Miller, *Robust Composition*](https://papers.agoric.com/assets/pdf/papers/robust-composition.pdf)).
The effects-as-capabilities line is valued for lightweight effect polymorphism
([Brachthäuser, Schuster & Ostermann, 2020](https://dl.acm.org/doi/10.1145/3428194)). Their costs,
as their users state them, are the costs of removing what everyone is used to. Ambient authority is
convenient, and passing an allocator by hand "doesn't exactly seem ideal"
([Wuyts](https://blog.yoshuawuyts.com/nesting-allocators)). Retrofitting is hard: `cap-std` cannot
sandbox code that still has `std::fs`, and Pony and Austral removed globals from the language, which
no established ecosystem can do after the fact. The effect-system designs pay in restrictions and
maturity: Effekt's capabilities cannot be returned or stored, and Scala's capture checking is, by its
own documentation, "still highly experimental and unstable". And the word itself is a cost, since the
senses are conflated so often that a 2003 paper set out to demolish the resulting myths
([Miller, Yee & Shapiro](https://papers.agoric.com/assets/pdf/papers/capability-myths-demolished.pdf)).

CGP's costs on this page are the properties it lacks. A reader who wants confinement gets none from
CGP alone and must bring `cap-std`, WASI, or a token type of their own. A reader who wants nested
scopes that shadow a binding gets a single flat context. A reader who wants a capability that can be
returned, stored, and revoked gets a field value with none of those semantics attached. Beyond these,
the ordinary costs apply: the wiring is code somebody writes, and the raw diagnostics are trait-solver
output over generated types. [`cargo cgp check`](/docs/cargo-cgp/check) leads with the root cause for
the classes it recognizes, and the tool is a v0.1.0-alpha that does not yet reshape every class. The
[Modularity Hierarchy](/docs/concepts/modularity-hierarchy) page weighs CGP's costs against the
alternatives.

## Where a capability system is the better choice

Where a program needs a capability-safe language, Pony, Austral, or Hardened JavaScript is the tool,
and where it needs sandboxed I/O in Rust, `cap-std` and WASI are. Where it needs privilege bits or
aliasing control, Linux capabilities and Rust's ownership already exist and CGP does not touch them.
CGP is the better tool where a program wants its implementations to state what they require and a
compiler to check that each context supplies it, on stable Rust, with the choice of implementation made
per application, and it combines with the object-capability libraries rather than competing with them.

## What to expect that differs

**CGP does not remove ambient authority.** A provider's bounds bound what it reaches through the
context, and nothing else. Rust's `std` and globals remain reachable from any body.

**A CGP bound is a requirement, not a token.** Any context with the right field satisfies it. Where
authority matters, it lives in the field's value, and a type with a private constructor is how Rust
makes such a value unforgeable.

**The authority structure is fixed per context type.** Field values vary at runtime; which providers
a context has, and what they require, does not. There is no runtime delegation, attenuation, or
revocation, and there is no nested scope.

**There is no escape check.** A context field can be cloned out and used elsewhere. Effekt's
second-class capabilities and Scala's capture sets prevent that; CGP does not.

**CGP does not "give Rust capabilities" in any single sense.** If the reader means implicit passing of
an allocator or a runtime, CGP's implicit arguments do that now, and the
[Rust proposals](./rust-language-proposals.md) page says what the contexts proposal would add. If the
reader means sandboxing, `cap-std` and WASI are the tools and CGP carries their handles. If the reader
means ownership tokens, Rust already has them and CGP can fix one as an abstract type. In CGP's own
vocabulary, a component defines a *trait*, a provider *requires* traits and *reads* fields, and a
context *supplies* what providers need; none of these is called a capability.

## Where to go next

- [Impl-side dependencies](/docs/concepts/impl-side-dependencies): the construct this page reads as a
  capability requirement.
- [Implicit arguments](/docs/concepts/implicit-arguments): reading a value from the context.
- [Algebraic effects](./algebraic-effects.md): the effects-as-capabilities fragment from the effect
  handler side.
- [Rust's own proposals](./rust-language-proposals.md): the contexts-and-capabilities proposal in
  full.

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
