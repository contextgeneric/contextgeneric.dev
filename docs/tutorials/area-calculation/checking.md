---
title: 'Checking and debugging CGP wiring errors'
sidebar_label: 'Checking and Debugging'
sidebar_position: 3
description: 'Wiring is lazy, so a mis-wired context still compiles. Catch the mistake where you made it with check_components!, and read the error with cargo cgp check.'
---

# Checking and Debugging

In the previous tutorial, we wired each shape context to the area calculator it needs with
`delegate_components!`. Every context we wrote happened to be wired correctly, which left one
question unasked: what happens when it is not?

In this tutorial we will make a wiring mistake on purpose and watch where Rust reports it. We will
find that it is reported nowhere useful, add `check_components!` to move the failure to the line where
we made it, and then read the same failure through `cargo cgp check`, which names the missing field in
plain English. By the end you will be able to wire a context and know immediately whether you got it
right.

We continue from the program the previous tutorial ended with, so the line numbers in the errors below
refer to that file rather than to yours.

## A mis-wired context still compiles

Here is the mistake. We wire `PlainCircle` to `RectangleAreaCalculator` — the provider that computes
`width * height` — instead of to `CircleAreaCalculator`:

```rust
delegate_components! {
    PlainCircle {
        AreaCalculatorComponent: RectangleAreaCalculator,
    }
}
```

A circle has a `radius` and no `width` or `height`, so this cannot possibly work. Run `cargo check`
anyway:

```text
Finished `dev` profile [unoptimized + debuginfo] target(s)
```

It compiles. Nothing warns, nothing fails, and the mistake sits in the code waiting.

This is worth understanding rather than working around, because it is not a bug. CGP wiring is
**lazy**: writing a `delegate_components!` entry records which provider a context uses, and Rust
checks whether that provider's requirements are met only when something actually asks the context to
do the work. Until then there is nothing to check against. The entry is a promise, and nobody has
called it in.

## Where the mistake surfaces instead

The promise is called in the moment we use the component. Add a line that asks the circle for its
area:

```rust
fn main() {
    let circle = PlainCircle { radius: 2.0 };
    println!("{}", circle.area());
}
```

Now `cargo check` fails, and this is what it says:

```text
error[E0599]: the method `area` exists for struct `PlainCircle`, but its trait bounds were not satisfied
  --> src/main.rs:61:27
   |
42 | pub struct PlainCircle {
   | ---------------------- method `area` not found for this struct because it doesn't satisfy
   |                        `PlainCircle: AreaCalculator<PlainCircle>` or `PlainCircle: CanCalculateArea`
...
61 |     println!("{}", circle.area());
   |                           ^^^^ this is an associated function, not a method
```

The full error runs to about thirty lines. Read what it tells us: the method is not available, and two
trait bounds are unsatisfied. Now read what it does not tell us. It does not mention `width`. It does
not mention `height`. It does not mention `RectangleAreaCalculator`, which is the thing we got wrong.
And it points at `main.rs` line 61, which is the line that is *correct* — asking a circle for its area
is a perfectly reasonable thing to do.

The mistake is fifteen lines earlier, in the wiring, and the compiler has no way to know that. It was
asked whether `PlainCircle` can do the work, found that it cannot, and said so at the place the
question was asked.

## Asking the question where you wired it

If the error appears wherever the component is used, then we should ask the question ourselves, at the
line where the wiring lives. That is what `check_components!` does:

```rust
check_components! {
    PlainCircle {
        AreaCalculatorComponent,
    }
}
```

This adds no behavior and generates nothing you will call. It is an assertion: *this context can use
this component*. Put it directly beneath the `delegate_components!` block it checks.

With that in place, `cargo check` reports the failure at the assertion instead:

```text
error[E0277]: the trait bound `PlainCircle: CanUseComponent<AreaCalculatorComponent>` is not satisfied
  --> src/main.rs:61:9
   |
61 |         AreaCalculatorComponent,
   |         ^^^^^^^^^^^^^^^^^^^^^^^ unsatisfied trait bound
   |
help: the trait `HasField<Symbol<5, Chars<'w', Chars<'i', Chars<'d', Chars<'t', Chars<'h', Nil>>>>>>>`
      is not implemented for `PlainCircle`
      but trait `HasField<Symbol<6, Chars<'r', Chars<'a', Chars<'d', Chars<'i', Chars<'u', Chars<'s', Nil>>>>>>>>`
      is implemented for it
...
note: required for `PlainCircle` to implement `RectangleArea`
note: required for `RectangleAreaCalculator` to implement `IsProviderFor<AreaCalculatorComponent, PlainCircle>`
```

Two things improved, and one did not.

The error now points at **our line**, the one we should be looking at, and the notes at the bottom
name the chain: `PlainCircle` was required to implement `RectangleArea`, which was required for
`RectangleAreaCalculator` to serve this context. That is the actual story of the mistake.

The cause is in there too, and this is the part that did not improve. That `Symbol<5, Chars<'w',
Chars<'i', ...>>>` is how CGP writes the field name `width` as a type, so that the compiler can
reason about field names at all. Read character by character it says `width`, and the second one says
`radius` — but you have to read it character by character, in the middle of a paragraph of angle
brackets, and the compiler abbreviates the display differently depending on how it is invoked.

## Reading it with `cargo cgp check`

Decoding type-level strings by hand is exactly the job a tool should do, and CGP ships one:

```bash
cargo install cargo-cgp
cargo cgp setup
```

The [installation page](/docs/cargo-cgp/installation) covers the other ways to get it, including the
Nix flake, and explains why `setup` is a separate step. `cargo cgp check` then runs the same check as
`cargo check` and rewrites the errors it recognizes. On the same broken program:

```text
error[E0277]: [CGP-E001] the consumer trait `CanCalculateArea` is not implemented for context `PlainCircle`
  --> src/main.rs:61:9
   |
61 |         AreaCalculatorComponent,
   |         ^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: root causes:
             - [CGP-E106] missing field `height` on `PlainCircle`
             - [CGP-E106] missing field `width` on `PlainCircle`
           this is required through the dependency chain:
             [CGP-E101] consumer trait impl `CanCalculateArea` for context `PlainCircle`
             └─ [CGP-E102] provider trait impl `AreaCalculator` with context `PlainCircle` for provider `RectangleAreaCalculator`
               └─ [CGP-E105] trait impl `RectangleArea` for `PlainCircle`
                 ├─ [CGP-E106] missing field `height` on `PlainCircle`
                 └─ [CGP-E106] missing field `width` on `PlainCircle`
```

This is the same failure a third time, and now it leads with the answer: `PlainCircle` is missing the
fields `width` and `height`. The tree beneath shows how the requirement arrived — through
`RectangleAreaCalculator`, which is the provider we wired by mistake.

One thing to keep in mind as you use it: `cargo cgp check` leads with the root cause for the classes
it recognizes, and the tool is a v0.1.0-alpha that does not yet reshape every class. When you meet an
error it has not rewritten, you will be reading the raw compiler output, which is why the previous
section is worth understanding rather than skipping.

## Fixing it

The error named the provider, so the fix is the line the error pointed at:

```rust
delegate_components! {
    PlainCircle {
        AreaCalculatorComponent: CircleAreaCalculator,
    }
}

check_components! {
    PlainCircle {
        AreaCalculatorComponent,
    }
}
```

`cargo run` now prints the circle's area:

```text
12.566370614359172
```

The check stays in the code. It costs nothing at runtime — it generates a trait and an impl and no
values — and it means the next person to touch this wiring finds out immediately if they break it.
Write one for every context you wire.

## How it works

*This section explains what `check_components!` generates. You can skip it and come back later.*

A check is an ordinary trait bound that you assert on purpose. Written by hand, the check above is
close to this:

```rust
pub trait CanUseAreaCalculator: CanCalculateArea {}

impl CanUseAreaCalculator for PlainCircle {}
```

The trait requires `CanCalculateArea`, and the impl claims `PlainCircle` satisfies it. If it does
not, the compiler rejects the impl — at the impl, which is a line we chose. Nothing calls
`CanUseAreaCalculator` and nothing needs to; writing the impl is the whole point, because writing it
is what forces the compiler to evaluate the bound.

`check_components!` adds precision about *why* a bound failed. It asserts a bound named
`CanUseComponent` rather than the consumer trait directly, and CGP's providers carry their
requirements in a marker called `IsProviderFor`. Those two are why the error above could name
`RectangleArea` and the missing field rather than stopping at "`CanCalculateArea` is not
implemented". You do not write either of them yourself — they exist so the failure can explain
itself.

## Summary

Wiring in CGP is lazy: a `delegate_components!` entry is checked when the component is used, not when
it is written, so a mis-wired context compiles until something asks it to work. The failure then
surfaces at the call site, which is the one place the mistake is not.

`check_components!` moves the question to the line you chose, directly beneath the wiring it checks,
and turns a latent mistake into a compile error where you can see it. `cargo cgp check` then reads the
result for you, naming the missing field and the chain that required it, for the error classes it
recognizes.

Neither is optional equipment once a context has more than a couple of components. Wire a context,
check it, and you will never be more than one line away from knowing whether it works.

---

*An AI agent wrote this page using the CGP knowledge base. Its code and every diagnostic it quotes
were produced by compiling and running the program. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
