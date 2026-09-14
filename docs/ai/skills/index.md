---
sidebar_position: 1
---

# CGP's Agent Skill

CGP's agent skill teaches coding assistants how to read, write, and debug CGP code. It is a set of
Markdown instructions in the [cgp-skills repository](https://github.com/contextgeneric/cgp-skills).
The pages in this section publish a snapshot of those files without changing their content.

## What it is for

The skill gives an assistant the CGP vocabulary and patterns it needs to work with the library.
It explains consumer and provider traits, component wiring, implementation dependencies, macro
expansions, and compiler errors. Without that guidance, an assistant may write a plain trait
implementation where a provider is needed or produce incorrect wiring.

You still need to review the assistant's code. The skill supplies instructions and examples; it
does not guarantee correct output or replace your understanding of the application.

## Using it

For a first look, download
[SKILL.md](https://raw.githubusercontent.com/contextgeneric/cgp-skills/refs/heads/main/cgp/SKILL.md)
and attach it to your assistant's conversation. The file introduces the core model and identifies
the reference files needed for particular tasks. Make those files available when the assistant needs
their detailed rules and examples.

For ongoing work, download the complete skill so the assistant can open its references as needed.
Clone the repository:

```sh
git clone https://github.com/contextgeneric/cgp-skills.git
```

Install the `cgp/` directory using your assistant's skill-loading instructions. Keep `cgp/SKILL.md`
and `cgp/references/` together so the relative links resolve. Cloning the repository downloads the
files; your assistant's setup determines how it discovers and loads them.

## Reading it yourself

The [tutorials](/docs/tutorials/hello) and [Concepts](/docs/concepts) pages are the starting points
for learning CGP. The skill is written for an assistant that needs dense rules and worked examples
for a specific task. Read it when you want that detail or want to see the instructions your
assistant receives.

## What is published here

The skill pages on this site are pinned to a particular repository revision. The repository may
contain newer changes, so use it as the authoritative source and report skill corrections there.
The site provides a place to browse the published snapshot; the repository supplies the files to
install.

---

*An AI agent wrote and revised this page using the CGP knowledge base. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
