import type {ReactNode} from 'react';
import Link from '@docusaurus/Link';
import Heading from '@theme/Heading';
import useBaseUrl from '@docusaurus/useBaseUrl';
import styles from './styles.module.css';

// The five headline features, in the order and wording the knowledge base's identity.md fixes.
// Each title links to the page that answers it, and each illustration is decorative beside its
// title, so its alt text is empty.
type Feature = {
  title: string;
  line: string;
  to: string;
  image: string;
};

const FEATURES: Feature[] = [
  {
    title: 'One Interface, Many Implementations',
    line: 'Implementations of one interface coexist, and each application names its choice.',
    to: '/docs/concepts/coherence',
    image: '/img/features/one-interface-many-implementations.webp',
  },
  {
    title: 'Zero-Cost Abstraction',
    line: 'Provider selection resolves at compile time into direct calls.',
    to: '/docs/comparisons/dynamic-dispatch',
    image: '/img/features/zero-cost-abstraction.webp',
  },
  {
    title: 'Type-Safe Wiring',
    line: 'A missing dependency fails the build once the wiring is checked.',
    to: '/docs/comparisons/dependency-injection',
    image: '/img/features/type-safe-wiring.webp',
  },
  {
    title: 'Abstract Over Every Dependency',
    line: 'Core logic names its error and runtime types, and each context supplies them.',
    to: '/docs/concepts/modular-error-handling',
    image: '/img/features/abstract-over-every-dependency.webp',
  },
  {
    title: 'Still Ordinary Rust',
    line: 'Adopt one component at a time; plain trait implementations still work.',
    to: '/docs/quickstart',
    image: '/img/features/still-ordinary-rust.webp',
  },
];

function FeatureCard({title, line, to, image}: Feature): ReactNode {
  return (
    <li className={styles.card}>
      <img className={styles.image} src={useBaseUrl(image)} alt="" width={180} height={180} />
      <Heading as="h3" className={styles.title}>
        <Link to={to}>{title}</Link>
      </Heading>
      <p className={styles.line}>{line}</p>
    </li>
  );
}

export default function FeatureCards(): ReactNode {
  return (
    <section className={styles.features} aria-label="The five headline features">
      <ul className={styles.cards}>
        {FEATURES.map((feature) => (
          <FeatureCard key={feature.title} {...feature} />
        ))}
      </ul>
    </section>
  );
}
