---
title: 'Videos about Context-Generic Programming'
sidebar_label: 'Videos'
sidebar_position: 1
description: "Recorded talks about Context-Generic Programming, starting with the RustLab 2025 talk on how CGP works within Rust's coherence rules."
---

# Videos

Some ideas are easier to follow when someone talks you through them. The talks on this page
introduce [Context-Generic Programming (CGP)](/docs/), which enables pluggable trait
implementations in Rust, and each comes with a note on what it covers and a link to its transcript
where one exists.

## RustLab 2025: How to stop fighting with coherence

<p>
<iframe width="560" height="315" style="max-width: 100%; height: auto; aspect-ratio: 16 / 9" src="https://www.youtube.com/embed/gXIfP-W9074?si=Q1qztb6J2PQ0b-jd" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>
</p>

Rust's coherence rules let a trait have only one implementation for any given type, so the
compiler always knows which one to use. The talk explains why the rules exist, then shows how CGP
works within them. If you prefer to read, its [full transcript](/blog/rustlab-2025-coherence) is on
the blog.

---

*An AI agent wrote this page using the CGP knowledge base. See
[How AI is used in this project](/docs/ai/disclaimer#documentation-and-reference-pages).*
