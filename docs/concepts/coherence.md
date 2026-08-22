---
sidebar_label: 'Bypassing coherence'
sidebar_position: 1
---

# Bypassing coherence

Rust allows one implementation of a trait per type. Why that rule exists, what it costs, and how CGP
writes many implementations and lets each type pick the one it wants.

This page answers *why can't Rust do this already?* It builds up what the trait system gives you, shows
what that costs when you want two implementations of one thing, and works through the move CGP makes,
which is smaller than it first appears and does not discard the rule. It closes on what that move costs in
turn. It is the page everything else in this section is downstream of.

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

There is no need to pass `Display` when `describe` is called. The caller writes `describe(&42)`, and the
compiler goes looking for `impl Display for i32`, finds it, and wires it up. That is dependency injection,
done by the type system at compile time, and it is so ordinary in Rust that Rust programmers rarely call
it that.

It is worth noticing that the compiler resolves the `T: Display` bound **transitively**. Suppose a pair
implements `Display` whenever both of its components do:

```rust
struct Pair<A, B>(A, B);

impl<A: Display, B: Display> Display for Pair<A, B> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}
```

Now `describe(&Pair("foo", 42u32))` asks the compiler for `Pair<&str, u32>: Display`, and to satisfy that
it resolves `&str: Display` and `u32: Display` in turn, without the caller naming either. The chain runs
as deep as the types do: add a requirement four layers down, and callers four layers up are unaffected.
That reach makes Rust's generics composable rather than a bookkeeping exercise.

## That only works because every lookup finds the same answer

Transitive resolution depends on something easy to overlook: when the compiler looks up `Display for i32`,
it must get the same answer no matter where it asks from. If one crate could see one implementation and
another crate a different one, the same generic function would mean different things depending on who
called it, and a program combining both would be incoherent: two halves compiled against
irreconcilable answers.

**Coherence is the property that guarantees this**, and Rust enforces it with two rules.

The **overlap rule** forbids two implementations that could both apply to the same type. Given a trait you
own and two blanket implementations (each a single `impl` written once for every type that meets a bound),
the compiler rejects the second:

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

`String` satisfies both bounds, so the compiler would have no principled way to choose. The rule exists
precisely to prevent the alternative: silently picking whichever implementation happens to be in scope.

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
the price of transitive resolution, and the trade is a good one. Any account of CGP that opens by
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

These pains are sharp enough that Rust developers work around them by hand, and at least one has
arrived independently at the pattern CGP is built on. The next section shows how CGP makes that pattern
first-class.

## The move: make `Self` a type you own

CGP's move is to split the trait in two and change what `Self` means on the side that implements it. One
definition becomes two traits. The **consumer trait** is the caller's view, `CanEncode` unchanged, so
`value.encode()` stays a method call. The **provider trait** carries the same method with the former
`Self` moved into a parameter, and each implementation targets a small type it declares for itself:

```rust
#[cgp_component(Encoder)]
pub trait CanEncode {
    fn encode(&self) -> Vec<u8>;
}
```

`#[cgp_component]` generates the provider trait `Encoder<Context>` from that one definition, where the
type being encoded is now the `Context` parameter. Each implementation targets a dummy struct of its own,
so the two implementations that clashed a moment ago can both be written:

```rust
#[cgp_impl(new EncodeAsText)]
#[uses(Display)]
impl Encoder {
    fn encode(&self) -> Vec<u8> { self.to_string().into_bytes() }
}

#[cgp_impl(new EncodeAsHex)]
#[uses(AsRef<[u8]>)]
impl Encoder {
    fn encode(&self) -> Vec<u8> { /* hex-encode self.as_ref() */ }
}
```

**Both compile.** `EncodeAsText` and `EncodeAsHex` are different `Self` types, so the two impls no longer
overlap, however many bounds they share, and any number more would compile too. The orphan rule does not
enter either, because that `Self` is always a struct this crate declared, which is how a crate adds a
capability to a type it did not define. Coherence only ever looked at `Self`, and `Self` is now a local
dummy struct, so there is nothing left for the rules to reject.

`#[cgp_impl]` lets a provider keep the consumer trait's `self` receiver, so inside `EncodeAsText` the
`self` is the value being encoded, and `#[uses(Display)]` records that this provider needs that value to
be `Display`. Neither rule was repealed; the implementations simply stopped being the kind of thing the
rules are about. [Consumer and provider traits](./consumer-and-provider-traits.md) works through the two
halves and how a call crosses between them.

## Coherence comes back, one type at a time

Making the implementations incoherent would be no use if it also made *using* them incoherent. A caller
still needs `value.encode()` to mean one definite thing.

So the second half brings coherence back at a smaller scale. A type chooses one provider for the
component, in a small wiring table:

```rust
delegate_components! { String  { EncoderComponent: EncodeAsText } }
delegate_components! { Vec<u8> { EncoderComponent: EncodeAsHex  } }
```

Now `String` implements `CanEncode` through `EncodeAsText` and `Vec<u8>` through `EncodeAsHex`, so a
`value.encode()` call resolves to the provider its type named. The two providers still overlap freely, yet
no call is ambiguous, because each type records its own choice.

That is the whole trade. **You want incoherence while writing implementations, and coherence while using
them.** While writing them, overlapping providers coexist with no global conflict. While using them, each
type names exactly one, and resolution is unambiguous.

**Coherence is not repealed; it is scoped.** That line is the whole page in a sentence.

How far the idea scales is a separate question. Here each type picks one provider, which is the simplest
shape. A type you define to stand for your own application can pick its own providers, and the same
capability can even resolve differently for the same data in two applications. Those are higher rungs of
the same ladder, worked out in [How much CGP to use](./modularity-hierarchy.md).

## What it costs

**It is more machinery than one trait.** A component is two traits, a marker type, and a wiring line per
type that uses it. For a capability with a single implementation that is pure overhead, and a plain trait
is the right tool. [`#[cgp_fn]`](/docs/reference/macros/cgp_fn) is the CGP construct for that case, and it
needs no wiring at all.

**A wired choice is local, not global.** This is a real limit rather than a technicality. When a program
genuinely needs one answer program-wide, per-type choice is the wrong shape and a coherent trait is safer.
Consider a single consistent `Ord` for a map key, where two orderings in one program would corrupt the
map. CGP does not make coherence optional; it moves where the single answer is decided.

**The choice is explicit, which is both the point and a cost.** Nothing is inferred. A type that has not
named a provider does not get a default, and adding a capability means adding a line. That is the property
the whole design rests on, and it is also more to write and read than a trait with one implementation.

**Wiring is checked lazily.** A table naming a provider whose requirements the wired type cannot meet
still compiles; the failure appears later, where the capability is used, and can be long.
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
