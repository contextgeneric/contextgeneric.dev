import type {ReactNode} from 'react';
import Link from '@docusaurus/Link';
import Heading from '@theme/Heading';
import useBaseUrl from '@docusaurus/useBaseUrl';
import styles from './styles.module.css';

// The first four headline features, in the order and with the titles the knowledge base's
// identity.md fixes; the hero's reassurance line carries the fifth, Still Ordinary Rust, so no card
// repeats it. Each card is one link to the page that answers it, and each illustration is decorative
// beside its title, so its alt text is empty.
type Feature = {
  title: string;
  body: ReactNode;
  to: string;
  image: string;
};

const FEATURES: Feature[] = [
  {
    title: 'One Interface, Many Implementations',
    body: (
      <>
        The two encoders above coexist because each one lives on its own provider type, and each
        application names the one it uses.
      </>
    ),
    to: '/docs/concepts/coherence',
    image: '/img/features/one-interface-many-implementations.webp',
  },
  {
    title: 'Zero-Cost Abstraction',
    body: (
      <>
        Provider selection resolves at compile time into a direct call, without a container or a
        vtable lookup.
      </>
    ),
    to: '/docs/comparisons/dynamic-dispatch',
    image: '/img/features/zero-cost-abstraction.webp',
  },
  {
    title: 'Type-Safe Wiring',
    body: (
      <>
        A provider states its needs, as <code>#[uses(Display)]</code> does above, and a{' '}
        <code>check_components!</code> assertion or a use of the component fails the build when a
        context lacks one.
      </>
    ),
    to: '/docs/comparisons/dependency-injection',
    image: '/img/features/type-safe-wiring.webp',
  },
  {
    title: 'Abstract Over Every Dependency',
    body: (
      <>
        Core logic names abstract error, runtime, and I/O types, each context supplies concrete ones,
        and the core builds for <code>no_std</code> where the chosen implementations allow.
      </>
    ),
    to: '/docs/concepts/modular-error-handling',
    image: '/img/features/abstract-over-every-dependency.webp',
  },
];

function FeatureCard({title, body, to, image}: Feature): ReactNode {
  return (
    <li>
      <Link className={styles.card} to={to}>
        <img className={styles.image} src={useBaseUrl(image)} alt="" width={180} height={180} />
        <span className={styles.text}>
          <Heading as="h3" className={styles.title}>
            {title}
          </Heading>
          <span className={styles.body}>{body}</span>
        </span>
      </Link>
    </li>
  );
}

// The cards sit two to a row, each illustration beside its text, and one to a row on a phone.
export default function FeatureCards(): ReactNode {
  return (
    <ul className={styles.cards}>
      {FEATURES.map((feature) => (
        <FeatureCard key={feature.title} {...feature} />
      ))}
    </ul>
  );
}
