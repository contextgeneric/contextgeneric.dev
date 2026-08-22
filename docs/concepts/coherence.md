---
sidebar_label: 'Bypassing coherence'
sidebar_position: 1
---

# Bypassing coherence

Rust allows one implementation of a trait per type. Why that rule exists, what it costs, and how CGP
writes many implementations and lets each context pick the one it wants.

This page answers *why can't Rust do this already?* It builds up what the trait system gives you, shows
what that costs when you want two implementations of one thing, and works through the move CGP makes,
which is smaller than it first appears and does not discard the rule. It closes on what that move costs in
turn. It is a long page, and it is the one everything else in this section is downstream of.

## The trait system is already a dependency-injection mechanism

Start with what Rust does well, because the rule that blocks this later is the same rule that makes
it work.

When you write a generic function with a bound, you are asking the compiler to find an implementation for
you:

```rust
pub fn describe<T: Display>(value: &T) -> String {
    format!("{value}")
}
```

Nobody passes `Display` in. The caller writes `describe(&42)` and the compiler goes looking for
`impl Display for i32`, finds it, and wires it up. That is dependency injection, done by the type system
at compile time, and it is so ordinary in Rust that it rarely gets called that.

The part worth noticing is that it works **transitively**. If `describe` calls something that itself
needs a bound, and that needs another, the compiler resolves the whole chain without a single caller
naming any of it. You can add a requirement four layers deep and callers four layers up are unaffected.
That property makes Rust's generics composable rather than a bookkeeping exercise.

## That only works because every lookup finds the same answer

Transitive resolution depends on something easy to overlook: when the compiler looks up `Display for i32`,
it must get the same answer no matter where it asks from. If one crate could see one implementation and
another crate a different one, the same generic function would mean different things depending on who
called it, and a program combining both would be incoherent: two halves compiled against
irreconcilable answers.

**Coherence is the property that guarantees this**, and Rust enforces it with two rules.

The **overlap rule** forbids two implementations that could both apply to the same type. Given a trait you
own and two blanket implementations, the compiler rejects the second:

```rust
pub trait CanEncode {
    fn encode(&self) -> Vec<u8>;
}

// Legal on its own.
impl<T: Display> CanEncode for T {
    fn encode(&self) -> Vec<u8> { self.to_string().into_bytes() }
}

// error[E0119]: conflicting implementations of trait `CanEncode`
impl<T: AsRef<[u8]>> CanEncode for T {
    fn encode(&self) -> Vec<u8> { self.as_ref().to_vec() }
}
```

`String` satisfies both bounds, so the compiler would have no principled way to choose. And "pick
whichever is in scope" is exactly the incoherence the rule exists to prevent.

The **orphan rule** forbids implementing a trait for a type unless your crate owns one of the two.
Neither `Display` nor `Vec<u8>` is yours, so this is rejected:

```rust
// error[E0117]: only traits defined in the current crate can be implemented
//               for types defined outside of the crate
impl Display for Vec<u8> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result { /* ... */ }
}
```

Without it, two unrelated crates could each add their own `Display for Vec<u8>`, and any program
depending on both would be unbuildable, with neither crate at fault.

**Both rules are correct.** They are not conservatism or an unfinished corner of the language; they are
what you pay for transitive resolution, and the trade is a good one. Any account of CGP that opens by
calling coherence a limitation has the argument backwards.

## What the guarantee costs

The price is that some perfectly reasonable code is unwritable, and the two rules cost you in different
ways.

The overlap rule costs you **alternative implementations**. In the encoding example above, the author knows
exactly which implementation they want for which type. The ambiguity is the compiler's, not theirs, and
there is no way to say so. The trait admits exactly one blanket implementation.

The orphan rule costs you **reach**. A crate that wants to add a capability to another crate's type has to
wrap it in a newtype and re-expose everything it still needs, which is boilerplate with no upside and a
type your callers now have to know about.

These are felt sharply enough that Rust developers build their own escape. The pattern is to stop
implementing the trait for the interesting type and implement it for a marker type you own instead, with a
helper trait tying the two together: three or four extra lines of plumbing, arrived at and written up
independently by a Rust developer working around the overlap rule. **If you have written that, you have
written CGP's central mechanism by hand**, and the rest of this page is about what it looks like made
first-class.

## The move: make `Self` a type you own

CGP's first move is to take the type coherence ranges over, the `Self` of the implementation, and make
it something the implementing crate always owns.

One trait definition becomes two. The **consumer trait** is the caller's view, keeping the ordinary `self`
receiver so that calling the capability is still a method call. The **provider trait** is the same
interface with `Self` moved out into an explicit parameter, and an implementation targets a small named
type of its own:

```rust
#[cgp_component(Encoder)]
pub trait CanEncodeValue<Value> {
    fn encode(&self, value: &Value) -> Vec<u8>;
}

#[cgp_impl(new EncodeAsText)]
impl<Value> Encoder<Value>
where
    Value: Display,
{
    fn encode(&self, value: &Value) -> Vec<u8> { value.to_string().into_bytes() }
}

#[cgp_impl(new EncodeAsHex)]
impl<Value> Encoder<Value>
where
    Value: AsRef<[u8]>,
{
    fn encode(&self, value: &Value) -> Vec<u8> { /* hex-encode the bytes */ }
}
```

**Both compile.** They carry the same two overlapping bounds the compiler rejected a moment ago:
`Display` and `AsRef<[u8]>`, still overlapping on `String`. There is no conflict, because each one's
`Self` is now a different type (`EncodeAsText`, `EncodeAsHex`) that this crate declared for the purpose.
Any number more would also compile. The orphan rule does not enter either, since that `Self` is always
local, which is how a crate adds a capability to a type it did not define.

Neither rule was repealed. The implementations simply stopped being the kind of thing the rules are
about. [Consumer and provider traits](./consumer-and-provider-traits.md) works through the mechanics of
the split and how a call finds its way across it.

One other thing changed in that listing, and it is worth naming rather than leaving you to spot it: the
value being encoded moved out of `Self` and into a `Value` parameter, so the trait is now
`CanEncodeValue<Value>` rather than `CanEncode`. That is a second move, not part of the first, and it
leaves `Self` free to be something other than the data. The next two sections cover what it is free to
be, and why that matters more than the parameter.

## Coherence comes back, one context at a time

Making implementations incoherent would be useless if it also made *use* incoherent. A caller still needs
`app.encode(&value)` to mean one definite thing.

So the second move reintroduces coherence at a smaller scale. Instead of one global choice per trait, each
**context**, a type you define to stand for one set of choices, names exactly one implementation in a
small table:

```rust
delegate_components! {
    ApiServer {
        open EncoderComponent;
        @EncoderComponent.String: EncodeAsText,
    }
}

delegate_components! {
    Firmware {
        open EncoderComponent;
        @EncoderComponent.String: EncodeAsHex,
    }
}
```

`ApiServer` encodes a `String` as its text and `Firmware` as hexadecimal:

```text
ApiServer: hi
Firmware:  6869
```

Within either context the choice is unambiguous, so resolution is coherent; between them it differs, with
no conflict, because the lookup is keyed on the context. Incoherence is only wanted when you are *writing*
implementations. When you use them, you want many independent local scopes, each internally consistent.

**Coherence is not repealed; it is scoped.** That line is the whole page in a sentence.

## Where the freedom actually comes from

Here is the part most easily misread, and it is worth slowing down for: **the escape happens when `Self`
becomes a type you own, not when a parameter appears.**

A scope is one context type, so the number of independent choices a program can make is the number of
context types it can define. Wire a capability onto `String`, which is data and not yours, and you get
exactly one answer for the whole program, because there is one `String`. Coherence was narrowed, not
removed. Wire it onto `ApiServer` and `Firmware`, types you declared, and you get as many answers as
you care to declare types.

Which is why the two contexts above are `ApiServer` and `Firmware` rather than `String` wired twice. The
second is impossible; the first is the point.

### The shape this depends on

That leaves one thing to build, because it is a shape vanilla Rust gives you no reason to have imagined:
**a type whose whole job is to stand for an application.**

Start from the fact that it is perfectly ordinary Rust. It is the arrangement from two sections ago with
the CGP taken back out: two application types, one value type, two encodings, and nothing exotic.

```rust
pub trait CanEncodeValue<Value> {
    fn encode(&self, value: &Value) -> Vec<u8>;
}

pub struct ApiServer;
pub struct Firmware;

impl CanEncodeValue<String> for ApiServer {
    fn encode(&self, value: &String) -> Vec<u8> { /* as text */ }
}

impl CanEncodeValue<String> for Firmware {
    fn encode(&self, value: &String) -> Vec<u8> { /* as hexadecimal */ }
}
```

This compiles, and it already does something useful: per-application encoding of the same type. Note that
both structs have **no fields at all**. `struct ApiServer;` is a complete and legitimate type here,
because its entire purpose is to be a name the implementations attach to. An empty struct with traits on it
is otherwise close to unreadable, which is why it is worth saying outright.

Now try to make it scale. Add a third value type, then a fourth, and you are hand-writing a body per
`(application, type)` pair. So you factor the shared logic into a blanket implementation, and the overlap
rule rejects it:

```rust
impl<V: Display> CanEncodeValue<V> for ApiServer { /* ... */ }

// error[E0119]: conflicting implementations of trait `CanEncodeValue<_>`
//               for type `ApiServer`
impl<V: AsRef<[u8]>> CanEncodeValue<V> for ApiServer { /* ... */ }
```

**So the shape is available and unrewarding.** It works, and nothing can be shared, so it stops paying off
at three types and nobody keeps it as a familiar technique. That is why "application context" usually
reads as an unfamiliar noun rather than a familiar arrangement: readers have not built it, because until
now there was no reason to.

This is the strongest form of the payoff, and it is worth stating precisely. **Coherence does not forbid
this shape; it makes it not worth building.** CGP's contribution is therefore constructive rather than
permissive: it does not merely escape a rule, it makes an available shape worth using. Named providers
replace the hand-written bodies, and a per-pair decision becomes a line in a table.

### Naming the two shapes

Two distinctions have been at work above. Naming them keeps "context" from becoming a word you cannot pin
down, and this is where readers most often lose track.

The first crossing had no signal at all. In the `CanEncode` listing further up, the wired type was **the
data**: `impl CanEncode for T` puts the thing being encoded in `Self`, exactly as `impl Display for String`
does, and that is the shape essentially every Rust trait is in. From *The move* onward the wired type has
been `ApiServer` or `Firmware`, types you define, holding no data and existing to carry choices. No
signature announced the change and no parameter marked it, yet `Self` stopped being data. The vocabulary
is a **value context** for the first and an **environmental context** for the second, and this is the
crossing that escapes coherence: you can define as many application types as you like, and there is only
one `String`.

The second crossing is visible in the code. A capability about `Self` is **self-targeted**, like
`CanEncode` as first written here. Moving the value into a parameter makes it **parameter-targeted**:
`CanEncodeValue<Value>` is about the `Value`, and `Self` only decides for it. An environmental context can
already choose for itself without that parameter; the parameter adds the ability to choose for a type you
*do not own*.

## What it costs

**It is more machinery than one trait.** A component is two traits, a marker type, and a wiring line per
context. For a capability with one implementation that is pure overhead, and a plain trait is the right
tool. [`#[cgp_fn]`](/docs/reference/macros/cgp_fn) is the CGP construct for that case, and it needs no
wiring at all.

**Uniqueness is per context, not global.** This is a real limit rather than a technicality. When a program
genuinely needs one answer program-wide, per-context choice is the wrong shape and a coherent trait is
safer. Consider a single consistent `Ord` for a map key, where two orderings in one program would corrupt
the map. CGP does not make coherence optional; it moves where the single answer is decided.

**The choice is explicit, which is both the point and a cost.** Nothing is inferred. A context that has
not named a provider does not get a default, and adding a capability means adding a line. That is the
property the whole design rests on, and it is also more to write and read than a trait with one
implementation.

**Wiring is checked lazily.** A table naming a provider whose requirements the context cannot meet still
compiles; the failure appears later, where the capability is used, and can be long.
[`check_components!`](/docs/reference/macros/check_components) forces it to the wiring line and names the
actual missing requirement, and
[`cargo cgp check`](https://github.com/contextgeneric/cargo-cgp) leads with the root cause for the classes
it recognizes. It is a `v0.1.0-alpha` that reshapes the core wiring errors rather than all of them.
[Checking your wiring](./check-traits.md) is the fuller account.

## Where to go next

[How much CGP to use](./modularity-hierarchy.md) is the page to read next if this argument landed, because
a reader who has just been told coherence can be escaped needs to hear immediately that most code should
not bother. It lays out the range from a plain trait to fully wired components and argues for climbing no
higher than a problem needs.

[Consumer and provider traits](./consumer-and-provider-traits.md) develops the trait split this page
introduced: what each half is for, and how a method call crosses between them.

To see it working rather than argued, the [Hello World tutorial](/docs/tutorials/hello) is five minutes and
one durable idea, and the [Area calculation series](/docs/tutorials/area-calculation/) builds the split up
from plain functions. For the constructs themselves,
[`#[cgp_component]`](/docs/reference/macros/cgp_component) generates the pair,
[`#[cgp_impl]`](/docs/reference/macros/cgp_impl) writes a provider, and
[`delegate_components!`](/docs/reference/macros/delegate_components) is the table.

The fullest public account of why coherence exists is the
[RustLab 2025 talk](/blog/rustlab-2025-coherence), which spends its first third establishing that the rule
is correct before working around it.

---

*This page was written by an AI agent from the CGP knowledge base and verified against the library's source — see [How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
