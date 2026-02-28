---
authors: [soares]
tags: [release]
---

# CGP has a new website, and why we moved from Zola to Docusaurus

If you have visited our website before, you might have noticed that the CGP project website has received a beautiful overhaul redesign with a fresh look and feel. This transformation comes from our migration away from [Zola](https://www.getzola.org/) to [Docusaurus](https://docusaurus.io/), a move driven by our commitment to significantly expand and improve the documentation for CGP.

<!-- truncate -->

## The journey to a better documentation home

As the CGP project continues to grow, we recognized that our documentation needs have evolved substantially. A comprehensive documentation website needs to support [many kinds of documentation](https://diataxis.fr/), including tutorials, guides, explanations, and references. These different documentation types often benefit from being organized into deeply nested levels that group related pages together.

While Zola is certainly a capable and highly customizable static site generator, it comes with some practical limitations for large-scale documentation projects. One key challenge is that Zola lacks a built-in way to automatically discover and list all pages in a sidebar, which meant I had to manually create and maintain links across different pages to help users navigate the documentation.

Additionally, customizing Zola's appearance requires significant investment in theme selection and template editing. Although many themes are available, I found that achieving the desired look and feel often required forking a theme and directly modifying its templates. This approach consumed considerable time that would have been better spent writing Rust code and documentation content for CGP.

## Discovering Docusaurus

Docusaurus offers a refreshing alternative with an excellent out-of-the-box experience that is specifically designed for large-scale documentation websites. Running `npx create-docusaurus` provides an immediately usable platform with all the essential features that CGP needed.

The customization I applied was intentionally minimal. I created a custom front page and adjusted the CSS to select custom colors for the website. Beyond these focused changes, everything simply works as intended right from the start.

My approach with Docusaurus is deliberately streamlined. I have no plans to install additional plugins or write React or JSX code. The default experience is genuinely pleasant, and the page rendering performance is good enough. Most importantly, Docusaurus allows me to concentrate my energy on creating and refining Markdown documentation files without worrying about customization or design challenges.

## The future possibilities of Rust-based documentation tools

Although we have adopted Docusaurus for our documentation platform today, I remain optimistic about the future of Rust-based website building tools. Rust has tremendous potential for creating powerful and efficient documentation websites that could eventually rival or exceed what we have today.

However, building such tools requires developers who possess both deep expertise in Rust and genuine experience in creating beautiful, user-friendly website designs. The reality is that most frontend developers work with JavaScript or TypeScript, while many Rust developers have not yet focused on building polished web experiences. A website's impact ultimately depends on how good it looks and feels, so it will take time to develop the necessary talent pool.

We can observe this dynamic by looking at GitHub contribution activity. Projects like Zola are primarily maintained by volunteers who contribute in their spare time, while tools like Docusaurus benefit from sustained funding and dedicated contributors. This difference in resources directly translates to the user experience and feature completeness.

Perhaps one day a Rust-based tool will emerge that matches Docusaurus in features, design, and community support. And perhaps, when CGP becomes mature enough, we could even use CGP itself to build such a tool. Right now, our focus remains on making CGP itself the best context-generic programming library it can be. Building web design tools and frameworks falls outside our current priorities.

## Leveraging AI to accelerate progress

One exciting aspect of developing this new website has been incorporating LLM assistance throughout the design and documentation process. This approach has allowed us to expand our website capabilities while working with the limited human resources available to the project.

You will notice that the design, text, and images on the [CGP front page](/) were largely created with the help of Claude Haiku and Gemini. Even the color theme was selected through LLM collaboration. While the results may not be perfect, they represent a substantial quality improvement over what I could have designed on my own.

This AI-assisted approach enabled us to build this new website in just 3 days, a timeline that would be difficult to achieve when working with a human designer. The asynchronous nature of LLM collaboration eliminated communication delays that typically slow down design work. Cost-wise, a few months of LLM subscriptions are far more economical than hiring a professional designer.

I want to be transparent about the trade-offs here. A professional human web designer would almost certainly create a more refined and cohesive website than what LLMs can produce today. However, since I was handling the design work myself before, the LLM assistance represents a genuine step forward in quality. Should the CGP project grow and achieve greater success, we would absolutely welcome the opportunity to collaborate with a skilled human designer to further elevate the website.

Beyond visual design, the text content throughout this website, including this blog post, has been reviewed and refined by LLMs to produce more polished and professional writing. This collaborative approach has helped transform my initial drafts into content that I believe better serves our audience.

## What comes next

With the website redesign now complete, we are excited about what lies ahead for the CGP project. CGP v0.7.0 will introduce support for using [**implicit parameters** in plain Rust function syntax](https://github.com/contextgeneric/cgp/issues/194), a feature that bridges the gap between plain Rust functions and fully context-generic code. Keep watching this website for the upcoming release announcement!