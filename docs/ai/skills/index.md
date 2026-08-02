---
sidebar_position: 1
---

# CGP's Agent Skill

CGP publishes an [agent skill](https://agentskills.io/): a set of Markdown files that teach an LLM
coding assistant how to read, write, and debug CGP code. It lives in the
[**cgp-skills** repository](https://github.com/contextgeneric/cgp-skills), and the pages in this
section are that skill, published here unchanged.

## What it is for

An assistant that has not met CGP treats it as ordinary Rust and gets the wiring wrong: it reaches for
a plain trait impl where a provider belongs, or writes a `delegate_components!` table it cannot then
explain. The skill supplies the missing vocabulary — consumer and provider traits, wiring, impl-side
dependencies — along with the code each macro expands to and how to read the errors CGP produces.

That is a narrow claim, and it is worth keeping narrow. The skill teaches the assistant, not you. It
does not make CGP simpler, and an assistant using it still writes code you have to review.

## Using it

The quickest way is to hand your assistant the main file. Download
[`SKILL.md`](https://raw.githubusercontent.com/contextgeneric/cgp-skills/refs/heads/main/cgp/SKILL.md)
and attach it to the context window of whichever model you use.

For everyday work, install the whole skill rather than the one file. Clone the repository and point
your assistant's skills directory at it, so the sub-skills load on demand instead of all at once:

```sh
git clone https://github.com/contextgeneric/cgp-skills.git
```

The layout follows the convention the tooling expects: `cgp/SKILL.md` is the entry point, and
`cgp/references/` holds one file per area — components, wiring, checking, handlers, and the rest. The
entry point is a complete primer on its own, and it tells the assistant which reference to open for
the construct in front of it.

## Reading it yourself

Nothing stops you, and the skill is written plainly enough to follow. But it is written *for a machine
reader*: it is dense, it repeats itself where repetition helps an assistant, and it assumes a reader
who wants exhaustive rules rather than a gentle path through them.

If you are learning CGP, the [Tutorials](/docs/tutorials/hello) start from a working program and the
[Concepts](/docs/concepts) pages take one idea at a time. Come back here when you want to see exactly
what your assistant has been told.

## What is published here

The pages below are the skill's own files, served from the `cgp-skills` repository as a submodule
pinned to one revision. Nothing here is retyped or summarised — a page is the skill's own bytes — but it
is a *snapshot*, and the repository moves first. Treat the repository as canonical, and raise any issue
or correction there.

---

*This page was written by an AI agent from the CGP knowledge base — see
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
