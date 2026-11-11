import {lightTheme as prismLight, darkTheme as prismDark} from './src/prism/themes';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

const config: Config = {
  title: 'CGP — Context-Generic Programming',
  tagline: 'A language extension for Rust, with pluggable trait implementations at compile-time',
  favicon: 'img/favicon.png',

  // The heading font is preloaded, because every page's title uses it. The SVG favicon switches to a
  // lighter red on a dark browser tab; the PNG stays as the fallback.
  // The theme-color tags match a phone's address bar to the navbar, using the values of `--cgp-bg`
  // in src/css/custom.css. Both follow the operating system's colour scheme, which is also the site's
  // default.
  headTags: [
    {tagName: 'link', attributes: {rel: 'icon', type: 'image/svg+xml', href: '/img/favicon.svg'}},
    {
      tagName: 'link',
      attributes: {
        rel: 'preload',
        href: '/fonts/space-grotesk/space-grotesk-latin.woff2',
        as: 'font',
        type: 'font/woff2',
        crossorigin: 'anonymous',
      },
    },
    {
      tagName: 'meta',
      attributes: {name: 'theme-color', media: '(prefers-color-scheme: light)', content: '#fdfaf7'},
    },
    {
      tagName: 'meta',
      attributes: {name: 'theme-color', media: '(prefers-color-scheme: dark)', content: '#161110'},
    },
  ],

  // Future flags, see https://docusaurus.io/docs/api/docusaurus-config#future
  future: {
    v4: true, // Improve compatibility with the upcoming Docusaurus v4
  },

  // Set the production url of your site here
  url: 'https://contextgeneric.dev',
  // Set the /<baseUrl>/ pathname under which your site is served
  // For GitHub pages deployment, it is often '/<projectName>/'
  baseUrl: '/',

  // GitHub pages deployment config.
  // If you aren't using GitHub pages, you don't need these.
  organizationName: 'contextgeneric', // Usually your GitHub org/user name.
  projectName: 'contextgeneric.dev', // Usually your repo name.

  onBrokenLinks: 'throw',
  onBrokenAnchors: 'throw',

  // Parse `.md` as CommonMark and `.mdx` as MDX, rather than treating every file
  // as MDX. The skill pages under `docs/ai/skills/` are symlinks into the
  // `cgp-skills` submodule — plain markdown written for coding agents, by a
  // repository that has no reason to know about MDX — and MDX would read an
  // autolink like `<https://example.com>` as a JSX tag and fail the build. No page
  // on this site uses MDX-only syntax (imports, exports, or JSX components), so
  // nothing else changes: admonitions and raw HTML work under both.
  markdown: {
    format: 'detect',
  },

  // Even if you don't use internationalization, you can use this field to set
  // useful metadata like html lang. For example, if your site is Chinese, you
  // may want to replace "en" with "zh-Hans".
  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  plugins: [
    // Keep webpack from resolving a symlink to its real path. The skill pages
    // under `docs/ai/skills/` are symlinks into the `cgp-skills` submodule, whose
    // real path sits outside `docs/`; resolved, the compiled module no longer
    // matches the metadata the docs plugin registered for the symlink, and every
    // such page fails to render with `Cannot read properties of undefined`.
    function resolveSymlinkedDocs() {
      return {
        name: 'resolve-symlinked-docs',
        configureWebpack() {
          return {resolve: {symlinks: false}};
        },
      };
    },
  ],

  presets: [
    [
      '@docusaurus/preset-classic',
      {
        docs: {
          showLastUpdateTime: true,
          sidebarPath: './sidebars.ts',
          // The `cgp-skills` submodule is checked out inside `docs/ai/skills/` so
          // the skill pages can symlink to it from next door. Its own files are the
          // source, not pages: the ones we publish are reached through those
          // symlinks, and the rest (README, AGENTS, sibling-projects) are not site
          // content at all.
          exclude: ['**/cgp-skills/**'],
          // Please change this to your repo.
          // Remove this to remove the "edit this page" links.
          editUrl:
            'https://github.com/contextgeneric/contextgeneric.dev/tree/main/',
        },
        blog: {
          showLastUpdateTime: true,
          showReadingTime: true,
          feedOptions: {
            type: ['rss', 'atom'],
            xslt: true,
          },
          // Please change this to your repo.
          // Remove this to remove the "edit this page" links.
          editUrl:
            'https://github.com/contextgeneric/contextgeneric.dev/tree/main/',
          // Useful options to enforce blogging best practices
          onInlineTags: 'warn',
          onInlineAuthors: 'warn',
          onUntruncatedBlogPosts: 'warn',
        },
        theme: {
          customCss: './src/css/custom.css',
        },
        pages: {},
        sitemap: {},
        svgr: {},
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    image: 'img/social-card.jpg',
    colorMode: {
      respectPrefersColorScheme: true,
    },
    announcementBar: {
      id: 'announcement',
      content:
        '🚀 <b>New Release:</b> <a href="/blog/v0.8.0-release">Group components with namespaces and paths in CGP v0.8.0</a> 🚀',
      // The bar's colours follow the theme through the tokens in src/css/custom.css.
      backgroundColor: 'var(--cgp-announce-bg)',
      textColor: 'var(--cgp-announce-text)',
      isCloseable: true,
    },
    navbar: {
      title: 'Context-Generic Programming',
      logo: {
        alt: 'Context-Generic Programming Logo',
        src: 'img/cgp-logo.svg',
        // The logo's red reaches only about 2:1 on the dark navbar, so the dark theme shows a lighter
        // tint.
        srcDark: 'img/cgp-logo-dark.svg',
      },
      items: [
        {to: '/docs/tutorials/hello', label: 'Tutorials', position: 'left'},
        {to: '/docs', label: 'Docs', position: 'left'},
        {to: '/blog', label: 'Blog', position: 'left'},
        {to: '/docs/ai/skills/', label: 'AI', position: 'left'},
        {
          href: 'https://github.com/contextgeneric/cgp',
          label: 'GitHub',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Docs',
          items: [
            {
              label: 'Introduction',
              to: '/docs',
            },
            {
              label: 'Tutorials',
              to: '/docs/category/tutorials',
            },
          ],
        },
        {
          title: 'Community',
          items: [
            {
              label: 'GitHub Discussions',
              href: 'https://github.com/orgs/contextgeneric/discussions',
            },
            {
              label: 'Discord',
              href: 'https://discord.gg/Hgk3rCw6pQ',
            },
            {
              label: 'Reddit',
              href: 'https://www.reddit.com/r/cgp/',
            },
          ],
        },
        {
          title: 'More',
          items: [
            {
              label: 'Blog',
              to: '/blog',
            },
            {
              label: 'GitHub',
              href: 'https://github.com/contextgeneric/cgp',
            },
          ],
        },
      ],
      copyright: `Licensed by <a href="https://maybevoid.com">MaybeVoid</a> under a <a href="https://creativecommons.org/licenses/by-sa/4.0/">Creative Commons Attribution-ShareAlike 4.0 International License</a>.`,
    },
    prism: {
      theme: prismLight,
      darkTheme: prismDark,
      // Prism's default bundle lacks these, so their fenced blocks would render as plain text. The
      // grammars ship in the `prismjs` package Docusaurus already installs. Scala extends Java's
      // grammar, so Java comes first.
      additionalLanguages: ['toml', 'bash', 'ebnf', 'haskell', 'ocaml', 'java', 'scala'],
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
