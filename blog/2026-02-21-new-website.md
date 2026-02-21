---
authors: [soares]
tags: [release]
---

# CGP has a new website, and why we moved from Zola to Docusaurus

If you have visited our website before, you might have noticed that the CGP project website has gone through an overhaul redesign with a fresh look. This is because we have now migrated away from [Zola](https://www.getzola.org/) to [Docusaurus](https://docusaurus.io/)!

<!-- truncate -->

## Why not Zola

It is a bit unfortunate that the CGP project has moved on from a Rust project (Zola) and is now relying on a TypeScript project (Docusaurus) for its website. But there is a simple reason behind this: we are expanding our effort to significantly improve the documentation for CGP. And to do that, we need to use a website builder tool that can support much larger scale documentation out of the box, and Docusaurus seems to fit all the requirements.

Although Zola is highly customizable, I (the CGP project maintainer) simply do not have enough time to work on improving the design by tweaking custom themes and templates. Furthermore, any time I spend on tweaking the website are missed opportunity of time that could be spent writing Rust code or documentation for CGP.

As I was planning to add more documentation to the website, I faced a tough choice of whether to continue using Zola to host the documentation. At a high level, a proper documentation website would need to support [many kinds of documentation](https://diataxis.fr/), such as tutorials, guides, explanation, and references. Within each kind of documentation, there would need to be further nested levels that can group multiple pages together.

With Zola, although it is possible to create deeply nested pages, there lacks a builtin way to automatically discover and list all pages in a sidebar. As a result, I often had to manually introduce links across different pages to show users what pages they can visit.

Furthermore, the default Zola theme is not very appealing. To make the project website look better, I had to manually search through the list of themes and choose a theme. But the worse is that regardless of which theme I chose, I eventually had to fork the theme and edit the theme templates directly to tweak the look of website. This not only wasted a lot of time, but it also introduced very tight coupling with the particular Zola theme that I had chosen.

## Why Docusaurus

On the other hand, Docusaurus provides exellent out of the box experience that is specifically designed for large-scale documentation websites. I just need to follow the instruction to run `npx create-docusaurus`, and I have gotten a default website that exactly has all the needs for CGP's website.

The main customization I did after creating the Docusaurus website is to create a custom front page, and tweak the CSS to choose custom colors for the website. Aside from that, everything you see just works out of the box.

I don't plan to install any additional plugins, nor do I plan to tweak the website by writing React or JSX code. Aside from that, the default experience of using Docusaurus is pretty pleasant, and the time for rendering the HTML pages are pretty reasonable. So to me, the main value that Docusaurus provides is that from now on, I can pretty much focus all my time writing .md documentation files without having to worry about customizing or designing the project website.

## Moving back to Rust in the future

Although we have moved to Docusaurus for now, I am hopeful that eventually one day in the future, the CGP project will be able to move back to a Rust-based website building tool.

There are a lot of potentials of how Rust would work better than JavaScript or TypeScript for building websites. But a significant barrier is that most of the frontend developers do not use Rust, and many Rust developers including myself lack expertise in building good looking websites.

Since the most important thing about a website is how good it looks, it will probably take time until we have developers who have both the expertise *and* also the time or funding required to build such tooling. By just looking at the GitHub contribution activity, it can show pretty clearly why projects like Zola lag so far behind Docusaurus: Zola is mainly worked on by volunteers, while Docusaurus has at least one contributor fully funded by Facebook, and do not have to worry about generating revenue.

Perhaps one day there will be a Rust project that offers the same good design and features similar to Docusaurus. And perhaps one day when CGP becomes mature enough, we can use CGP to build such tool. But for now, the CGP project's priority is not on web designs and building front end frameworks. So we will live with whatever that is necessary for the project to move on.

## AI-Assisted design and documentation

A big part of the new website development effort will incorporate LLM assistance, so that we can expand the website with the limited human resource available.

You might notice that the design, text, and images of the [CGP front page](/) are largely produced by Claude Haiku and Gemini. The color theme of the website is also chosen by the LLM. Although the designs might not be perfect, I'd say that they are still significantly better than how I could have designed them on my own.

The AI-assisted design also allows me to quickly setup this new website in just 3 days. The communication delay of working with a human designer would probably take much longer, much less to say that the cost would probably be enough for a few months worth of LLM subscription.

That said, I certainly won't deny that a human professional web designer could probably produce a much better and consistent website design than the LLM. It is just that what I am replacing here is myself as the designer, so obviously the LLM fares much better. If the CGP project grows and become more successful, I'm certainly open to getting a human designer to help further improve the website design.

Aside from this, the text on this website, including this blog post have all gone through reviews and revisions by LLM. This help me produce much more professional writing that I could have written on my own.

## Up next: CGP v0.6.2 release

With the website redesign over, here is a sneak preview of what is coming in the next few days: CGP v0.6.2 will be adding support for using [**implicit parameters** in plain Rust function syntax](https://github.com/contextgeneric/cgp/issues/194). So stay tuned on this website and watch out for the coming release!