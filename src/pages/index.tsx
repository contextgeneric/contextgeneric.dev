import {useState, type ReactNode} from 'react';
import clsx from 'clsx';
import Link from '@docusaurus/Link';
import Head from '@docusaurus/Head';
import useBaseUrl from '@docusaurus/useBaseUrl';
import Layout from '@theme/Layout';
import Heading from '@theme/Heading';
import CodeBlock from '@theme/CodeBlock';
import FeatureCards from '@site/src/components/HomepageFeatures';

import styles from './index.module.css';

// The front page is written as its own markup and stylesheet on the site theme's tokens, inside the
// Docusaurus layout. Its copy, code, and art follow the knowledge base's homepage guide.

// The page's code. `example-code/tests/front_page.rs` compiles each block and compares the text
// between each pair of `front-page:` markers with the same markers in its own sources, so a change
// here needs the same change there. The knowledge base's homepage guide specifies the blocks.

// front-page:rejected-block:begin
const REJECTED_BLOCK = `// Rust allows only one of these. \`String\` is both \`Display\` and
// \`AsRef<[u8]>\`, so the compiler has no principled way to choose.
pub trait CanEncode {
    fn encode(&self) -> Vec<u8>;
}

impl<T: Display> CanEncode for T {
    fn encode(&self) -> Vec<u8> { self.to_string().into_bytes() }
}

impl<T: AsRef<[u8]>> CanEncode for T {          // error[E0119]
    fn encode(&self) -> Vec<u8> { self.as_ref().to_vec() }
}`;
// front-page:rejected-block:end

// The compiler's error for the rejected block, from its compile-fail snapshot, without the location
// line, which names the fixture's path. The block starts at the fixture's first line, so the line
// numbers here are the ones the page shows beside the block.
// front-page:rejected-diagnostic:begin
const REJECTED_DIAGNOSTIC = `error[E0119]: conflicting implementations of trait \`CanEncode\`
   |
 7 | impl<T: Display> CanEncode for T {
   | -------------------------------- first implementation here
...
11 | impl<T: AsRef<[u8]>> CanEncode for T {          // error[E0119]
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ conflicting implementation`;
// front-page:rejected-diagnostic:end

// front-page:cgp-block:begin
const CGP_BLOCK = `#[cgp_component(Encoder)]
pub trait CanEncode {
    fn encode(&self) -> Vec<u8>;
}

#[cgp_impl(new EncodeAsText)]
#[uses(Display)]
impl Encoder {
    fn encode(&self) -> Vec<u8> { self.to_string().into_bytes() }
}

#[cgp_impl(new EncodeAsBytes)]
#[uses(AsRef<[u8]>)]
impl Encoder {
    fn encode(&self) -> Vec<u8> { self.as_ref().to_vec() }
}

// Each type names the implementation it uses:
delegate_components! { u64     { EncoderComponent: EncodeAsText } }
delegate_components! { String  { EncoderComponent: EncodeAsText } }
delegate_components! { Vec<u8> { EncoderComponent: EncodeAsBytes } }`;
// front-page:cgp-block:end

const INSTALL_COMMAND = 'cargo add cgp';

// The install line copies its command, so a reader can paste it without selecting the text. A
// browser without the clipboard API, or one that refuses it, leaves the button unchanged, and the
// command stays selectable by hand.
function InstallLine(): ReactNode {
  const [copied, setCopied] = useState(false);
  const copy = () => {
    navigator.clipboard
      ?.writeText(INSTALL_COMMAND)
      .then(() => {
        setCopied(true);
        setTimeout(() => setCopied(false), 1500);
      })
      .catch(() => {});
  };
  return (
    <div className={styles.install}>
      <code>
        <span className={styles.prompt} aria-hidden="true">
          $
        </span>{' '}
        {INSTALL_COMMAND}
      </code>
      <button type="button" className={styles.copy} onClick={copy} aria-live="polite">
        {copied ? 'Copied' : 'Copy'}
      </button>
    </div>
  );
}

function Hero(): ReactNode {
  return (
    <header className={styles.hero}>
      <div className={clsx(styles.container, styles.heroGrid)}>
        <div>
          <Heading as="h1" className={styles.heroTitle}>
            Context-Generic Programming
          </Heading>
          <p className={styles.heroTagline}>
            A language extension for Rust, with pluggable trait implementations at compile-time.
          </p>
          <p className={styles.heroReassurance}>
            Still ordinary Rust: a library on the stable toolchain, with provider selection resolved
            at compile time, adopted one trait at a time.
          </p>
          <InstallLine />
          <div className={styles.heroLinks}>
            <Link className={styles.primaryButton} to="/docs/quickstart">
              Quickstart
            </Link>
            <Link className={styles.secondaryButton} to="#costs">
              Costs and maturity
            </Link>
            <iframe
              className={styles.starButton}
              src="https://ghbtns.com/github-btn.html?user=contextgeneric&amp;repo=cgp&amp;type=star&amp;count=true"
              width={110}
              height={20}
              title="Star CGP on GitHub"
            />
          </div>
        </div>
        <div className={styles.heroArtFrame}>
          <img
            className={styles.heroArt}
            src={useBaseUrl('/img/cgp-hero.webp')}
            alt="The Context-Generic Programming logo"
            width={280}
            height={202}
          />
        </div>
      </div>
    </header>
  );
}

type CodeWindowProps = {
  label: string;
  verdict: 'rejected' | 'accepted';
  metastring: string;
  code: string;
  diagnostic?: string;
};

// A code block framed as an editor window. The title bar states the verdict with a mark as well as
// a colour, and the highlighted lines carry a gutter glyph, so the two blocks differ for a reader who
// cannot tell red from green.
function CodeWindow({label, verdict, metastring, code, diagnostic}: CodeWindowProps): ReactNode {
  return (
    <figure className={clsx(styles.window, styles[verdict])}>
      <figcaption className={styles.windowBar}>
        <span className={styles.verdictMark} aria-hidden="true">
          {verdict === 'rejected' ? '✗' : '✓'}
        </span>
        {label}
      </figcaption>
      <div className={styles.windowCode}>
        <CodeBlock language="rust" metastring={metastring}>
          {code}
        </CodeBlock>
      </div>
      {diagnostic && (
        <pre className={styles.diagnostic}>{diagnostic}</pre>
      )}
    </figure>
  );
}

function CodeBand(): ReactNode {
  return (
    <section className={styles.codeBand} aria-label="The same two implementations, without and with CGP">
      <div className={clsx(styles.container, styles.codeColumns)}>
        <CodeWindow
          label="Rust accepts only one of these"
          verdict="rejected"
          metastring="showLineNumbers {11}"
          code={REJECTED_BLOCK}
          diagnostic={REJECTED_DIAGNOSTIC}
        />
        <CodeWindow
          label="With CGP, both compile"
          verdict="accepted"
          metastring="{1,6-7,12-13,18-21}"
          code={CGP_BLOCK}
        />
      </div>
      <div className={clsx(styles.container, styles.caption)}>
        <p className={styles.captionReason}>
          Both implementations compile, because each one now has a name, and each type names the one
          it uses, so a call to <code>encode()</code> is as unambiguous as it ever was.
        </p>
        <p className={styles.captionQuote}>Coherence is not repealed; it is scoped.</p>
      </div>
    </section>
  );
}

function WhyOnce(): ReactNode {
  return (
    <section className={clsx(styles.section, styles.sectionTinted)}>
      <div className={clsx(styles.container, styles.split)}>
        <Heading as="h2" className={styles.sectionTitle}>
          Why Rust allows this only once
        </Heading>
        <div className={styles.prose}>
          <p>
            The compiler rejects the second implementation for a good reason. Rust's trait system
            finds implementations for you: when a generic implementation requires{' '}
            <code>T: Display</code>, the compiler finds the <code>Display</code> implementation, and
            everything that implementation needs in turn, without the caller naming any of it. That
            search works only because every lookup finds the same answer, and the overlap and orphan
            rules, together called coherence, make sure it does. So coherence is a guarantee Rust
            gives you, not a defect.
          </p>
          <p>
            Without CGP, then, a second implementation needs a workaround: a newtype for each case,
            or a marker struct and a helper trait written by hand.{' '}
            <Link to="/docs/concepts/coherence">Bypassing coherence</Link> explains the rules and
            what they cost in practice.
          </p>
          <p>
            CGP keeps coherence and moves the choice instead. Each implementation gets its own
            provider type, owned by the crate that writes it, so neither the overlap rule nor the
            orphan rule rejects it. Each wired type then names exactly one provider, so every lookup
            still finds a single answer.
          </p>
          <p>
            The types wired above are data, though, so each of them gets one choice for the whole
            program. To let each application choose, you wire a type you define to stand for the
            application, which CGP calls a <strong>context</strong>, and because a context is your
            own type, you can define as many as you need.{' '}
            <Link to="/docs/concepts/consumer-and-provider-traits">Consumer and provider traits</Link>{' '}
            shows an application and its test choosing different providers.
          </p>
        </div>
      </div>
    </section>
  );
}

function Features(): ReactNode {
  return (
    <section className={styles.section}>
      <div className={styles.container}>
        <Heading as="h2" className={styles.sectionTitle}>
          What that gives you
        </Heading>
        <p className={styles.lead}>
          Naming each implementation and letting each application choose give CGP its main features:
        </p>
        <FeatureCards />
        <p className={styles.sectionNote}>
          The <Link to="/docs/overview">Overview</Link> describes each feature in more detail.
        </p>
      </div>
    </section>
  );
}

type Breadth = {title: string; to: string; body: ReactNode};

const BREADTH: Breadth[] = [
  {
    title: 'Abstract types',
    to: '/docs/concepts/abstract-types',
    body: (
      <>
        A context chooses its own types, so an error or runtime type stops being a parameter that
        every layer declares.
      </>
    ),
  },
  {
    title: 'Extensible records and variants',
    to: '/docs/concepts/extensible-records',
    body: (
      <>
        A struct or enum that opts in with a derive exposes its fields and variants to generic code
        that builds and matches them.
      </>
    ),
  },
  {
    title: 'Handlers',
    to: '/docs/concepts/handlers',
    body: (
      <>
        Small steps, synchronous or async, compose into a pipeline whose steps you can change one at
        a time.
      </>
    ),
  },
];

function MoreThanTraits(): ReactNode {
  return (
    <section className={clsx(styles.section, styles.sectionTinted)}>
      <div className={styles.container}>
        <Heading as="h2" className={styles.sectionTitle}>
          More than trait implementations
        </Heading>
        <p className={styles.lead}>CGP applies the same mechanism to types, data, and computation.</p>
        <ul className={styles.tileGrid}>
          {BREADTH.map(({title, to, body}) => (
            <li key={title}>
              <Link className={styles.tile} to={to}>
                <span className={styles.tileTitle}>{title}</span>
                <span className={styles.tileBody}>{body}</span>
              </Link>
            </li>
          ))}
        </ul>
      </div>
    </section>
  );
}

type Cost = {label: string; body: ReactNode};

// The costs as a two-column list of labels and statements. The `cargo cgp check` sentence is the knowledge base's canonical
// qualification, copied word for word.
const COSTS: Cost[] = [
  {
    label: 'More declarations',
    body: <>Components, providers, and wiring add code that a plain trait does not need.</>,
  },
  {
    label: 'Compile-time work',
    body: <>The macros and the type checking add work at compile time.</>,
  },
  {
    label: 'Verbose raw errors',
    body: (
      <>
        Raw wiring errors are long, but{' '}
        <Link to="/docs/cargo-cgp">
          <code>cargo cgp check</code>
        </Link>{' '}
        leads with the root cause for the classes it recognizes, and the tool does not yet reshape
        every class.
      </>
    ),
  },
  {
    label: 'A young ecosystem',
    body: (
      <>
        The ecosystem is small and community support is limited, so on a mission-critical project,
        expect to investigate some problems on your own.{' '}
        <code>cargo-cgp</code> is itself an early release.
      </>
    ),
  },
  {
    label: 'Learning time',
    body: (
      <>
        Learning the patterns takes time. CGP's <Link to="/docs/ai/skills">agent skill</Link> helps
        an assistant with the vocabulary, the diagnostics, and the wiring, which makes these costs
        smaller, but code it writes still needs your review.
      </>
    ),
  },
];

function Costs(): ReactNode {
  return (
    <section className={styles.section}>
      <div className={styles.container}>
        {/* The heading and its lead sit in a column beside the costs, as section 1's title does, so
            the list fills the panel's width at a readable line length. */}
        <div className={clsx(styles.costPanel, styles.costSplit)}>
          <div>
            <Heading as="h2" id="costs" className={styles.sectionTitle}>
              What it costs
            </Heading>
            <p className={styles.costLead}>
              For a trait with one implementation, a plain trait is the right tool, because CGP has
              these costs:
            </p>
          </div>
          <div className={styles.prose}>
            {/* A description list rather than a table: each row pairs one label with one statement, and
                the rows share no other columns. */}
            <dl className={styles.costTable}>
              {COSTS.map(({label, body}) => (
                <div key={label} className={styles.costRow}>
                  <dt>{label}</dt>
                  <dd>{body}</dd>
                </div>
              ))}
            </dl>
            <p className={styles.costAdvice}>
              So try CGP first on a small part of a project whose implementation choices already
              vary. <Link to="/docs/concepts/modularity-hierarchy">Modularity Hierarchy</Link> shows
              when a plain trait is enough.
            </p>
          </div>
        </div>
      </div>
    </section>
  );
}

type Route = {reader: string; to: string; link: string; reason: ReactNode};

// Each route card is one stretched link to its main page, with an optional second link above the
// stretched area, because a link inside a link is invalid.
const ROUTES: Route[] = [
  {
    reader: 'Try it',
    to: '/docs/quickstart',
    link: 'Quickstart',
    reason: (
      <>
        Install CGP and run one program, then the{' '}
        <Link to="/docs/tutorials/hello">Hello World tutorial</Link>.
      </>
    ),
  },
  {
    reader: 'Understand the idea',
    to: '/docs/concepts/coherence',
    link: 'Bypassing coherence',
    reason: (
      <>
        The argument in full, then{' '}
        <Link to="/docs/concepts/consumer-and-provider-traits">Consumer and provider traits</Link>.
      </>
    ),
  },
  {
    reader: 'Decide whether it fits',
    to: '/docs/concepts/modularity-hierarchy',
    link: 'Modularity Hierarchy',
    reason: (
      <>
        When a simpler tool is enough, and the{' '}
        <Link to="/docs/projects">projects built with CGP</Link>.
      </>
    ),
  },
  {
    reader: 'Go deeper',
    to: '/docs/overview',
    link: 'Overview',
    reason: (
      <>
        Every feature, then the <Link to="/docs/comparisons">comparisons</Link> and the{' '}
        <Link to="/blog">blog</Link>.
      </>
    ),
  },
  {
    reader: 'Help build it',
    to: '/docs/contribute',
    link: 'Contribute',
    reason: <>Join the discussions, or improve the code and the documentation.</>,
  },
];

function WhereNext(): ReactNode {
  return (
    <section className={clsx(styles.section, styles.sectionTinted)}>
      <div className={styles.container}>
        <Heading as="h2" className={styles.sectionTitle}>
          Where to go next
        </Heading>
        <p className={styles.lead}>
          Pick the next step that fits what you want from CGP.
        </p>
        <ul className={styles.routeGrid}>
          {ROUTES.map(({reader, to, link, reason}) => (
            <li key={reader} className={styles.route}>
              <span className={styles.routeReader}>{reader}</span>
              <Link className={styles.routeLink} to={to}>
                {link}
              </Link>
              <span className={styles.routeReason}>{reason}</span>
            </li>
          ))}
        </ul>
      </div>
    </section>
  );
}

export default function Home(): ReactNode {
  return (
    <Layout
      description="A language extension for Rust, with pluggable trait implementations at compile-time. Give one interface several implementations and choose per application.">
      {/*
        The front page names the project first, which Docusaurus's own title formatter cannot do:
        passing `title` to Layout renders `{title} | {siteTitle}`, so the project name could only ever
        come last. Setting the tag here replaces that entirely, and og:title is set with it so a
        shared link carries the same words.
      */}
      <Head>
        <title>
          Context-Generic Programming (CGP) - Pluggable trait implementations for Rust
        </title>
        <meta
          property="og:title"
          content="Context-Generic Programming (CGP) - Pluggable trait implementations for Rust"
        />
      </Head>
      <main className={styles.page}>
        <Hero />
        <CodeBand />
        <WhyOnce />
        <Features />
        <MoreThanTraits />
        <Costs />
        <WhereNext />
      </main>
    </Layout>
  );
}
