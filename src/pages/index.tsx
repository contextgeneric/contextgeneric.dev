import type {ReactNode} from 'react';
import clsx from 'clsx';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import useBaseUrl from '@docusaurus/useBaseUrl';
import Layout from '@theme/Layout';
import HomepageFeatures from '@site/src/components/HomepageFeatures';
import Heading from '@theme/Heading';
import CodeBlock from '@theme/CodeBlock';

import styles from './index.module.css';

function HeroBanner() {
  return (
    <div className={styles.hero} data-theme="light">
      <div className={styles.heroOuter}>
        <img
          alt="Context-Generic Programming"
          className={styles.heroLogo}
          src={useBaseUrl('/img/cgp-hero.png')}
          width="300"
          height="300"
        />
        <div className={styles.heroInner}>
          <Heading as="h1" className={styles.heroProjectTagline}>
            <span className={styles.heroTitleTextHtml}>
              Build <b>modular</b> Rust applications with <b>zero-cost</b> abstractions
            </span>
          </Heading>
          <div className={styles.indexCtas}>
            <Link className={clsx("button button--primary button--lg", styles.getStartedButton)} to="/docs">
              Get Started
            </Link>
            <Link
              className={clsx("button button--secondary button--lg margin-left--md", styles.tutorialButton)}
              to="/docs/tutorials/hello">
              Tutorial - 10 min ⏱️
            </Link>
            <span className={styles.indexCtasGitHubButtonWrapper}>
              <iframe
                className={styles.indexCtasGitHubButton}
                src="https://ghbtns.com/github-btn.html?user=contextgeneric&amp;repo=cgp&amp;type=star&amp;count=true&amp;size=large"
                width={160}
                height={30}
                title="GitHub Stars"
              />
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}

function CodeExampleSection() {
  return (
    <div className={clsx(styles.section, styles.sectionAlt)}>
      <div className="container">
        <div className="row">
          <div className="col col--6">
            <Heading as="h2">Bypass Coherence Restrictions</Heading>
            <p>
              CGP enables you to write <b>overlapping</b> and <b>orphan</b> implementations
              of any trait, breaking free from Rust's coherence rules while maintaining
              type safety.
            </p>
            <p>
              Annotate any trait with <code>#[cgp_component]</code>, write named implementations
              with <code>#[cgp_impl]</code>, and selectively enable them using{' '}
              <code>delegate_components!</code>.
            </p>
          </div>
          <div className="col col--6">
            <div className={styles.codeExample}>
              <CodeBlock language="rust">
{`#[cgp_component(HashProvider)]
pub trait Hash { ... }

#[cgp_impl(HashWithDisplay)]
impl HashProvider
where
    Self: Display,
{ ... }

delegate_components! {
    MyData {
        HashProviderComponent: HashWithDisplay,
    }
}`}
              </CodeBlock>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function ProblemsSection() {
  return (
    <div className={styles.section}>
      <div className="container">
        <Heading as="h2" className="text--center margin-bottom--xl">
          Common Rust Problems, Solved
        </Heading>
        <div className="row">
          <div className="col col--4">
            <div className={styles.problemCard}>
              <Heading as="h3">⚡ No More Monolithic Traits</Heading>
              <p>
                Break down large traits into small, focused components.
                Compose features without trait bounds propagating everywhere.
              </p>
            </div>
          </div>
          <div className="col col--4">
            <div className={styles.problemCard}>
              <Heading as="h3">🔄 Overlapping Implementations</Heading>
              <p>
                Implement traits in multiple ways without newtype wrappers.
                Choose the right implementation for your context.
              </p>
            </div>
          </div>
          <div className="col col--4">
            <div className={styles.problemCard}>
              <Heading as="h3">🎯 Decouple Dependencies</Heading>
              <p>
                Write core logic independent of error handling, async runtimes,
                or serialization libraries. Wire them up at the edges.
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function CallToActionSection() {
  return (
    <div className={clsx(styles.section, styles.sectionAlt)}>
      <div className="container text--center">
        <Heading as="h2">Ready to Get Started?</Heading>
        <p className="margin-bottom--lg">
          CGP is in active development and perfect for early adopters who want to
          experiment with advanced Rust patterns.
        </p>
        <div className={styles.buttons}>
          <Link className="button button--primary button--lg" to="/docs">
            Read the Docs
          </Link>
          <Link
            className="button button--secondary button--lg margin-left--md"
            to="/docs/contribute">
            Contribute to CGP
          </Link>
        </div>
      </div>
    </div>
  );
}

export default function Home(): ReactNode {
  const {siteConfig} = useDocusaurusContext();
  return (
    <Layout
      title={siteConfig.tagline}
      description="Context-Generic Programming (CGP) - A modular programming paradigm for Rust">
      <main>
        <HeroBanner />
        <HomepageFeatures />
        <CodeExampleSection />
        <ProblemsSection />
        <CallToActionSection />
      </main>
    </Layout>
  );
}
