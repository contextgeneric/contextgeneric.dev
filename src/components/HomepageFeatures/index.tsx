import type {ReactNode} from 'react';
import clsx from 'clsx';
import Heading from '@theme/Heading';
import styles from './styles.module.css';

type FeatureItem = {
  title: string;
  image: string;
  description: ReactNode;
};

const FeatureList: FeatureItem[] = [
  {
    title: 'Modular Component System',
    image: '/img/features/modular-component-system.png',
    description: (
      <>
        Decouple interface definitions from implementations using provider traits.
        Write multiple context-generic providers that can coexist without conflicts.
      </>
    ),
  },
  {
    title: 'Highly Expressive Macros',
    image: '/img/features/highly-expressive-macros.png',
    description: (
      <>
        Write abstract programs generic over contexts without managing long lists
        of generic parameters. Achieve expressiveness rivaling dynamic languages.
      </>
    ),
  },
  {
    title: 'Type-Safe Composition',
    image: '/img/features/type-safe-composition.png',
    description: (
      <>
        All component wiring is verified at compile time. Missing dependencies are
        caught early, with no runtime errors possible from CGP constructs.
      </>
    ),
  },
  {
    title: 'No-Std Friendly',
    image: '/img/features/no-std-friendly.png',
    description: (
      <>
        Build fully abstract programs without concrete dependencies. Deploy to
        embedded systems, kernels, WebAssembly, or symbolic execution platforms.
      </>
    ),
  },
  {
    title: 'Zero-Cost Abstraction',
    image: '/img/features/zero-cost-abstraction.png',
    description: (
      <>
        All CGP operations happen at compile time using Rust's type system.
        No runtime overhead, upholding Rust's zero-cost abstraction guarantee.
      </>
    ),
  },
  {
    title: 'Bypass Coherence Rules',
    image: '/img/features/bypass-coherence-rules.png',
    description: (
      <>
        Enable overlapping and orphan trait implementations without restriction.
        Define implementations for types you don't own and traits you don't control.
      </>
    ),
  },
];

function Feature({title, image, description}: FeatureItem) {
  return (
    <div className={clsx('col col--4')}>
      <div className="text--center">
        <img className={styles.featureSvg} src={image} alt={title} />
      </div>
      <div className="text--center padding-horiz--md">
        <Heading as="h3">{title}</Heading>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures(): ReactNode {
  return (
    <section className={styles.features}>
      <div className="container">
        <Heading as="h2" className={clsx(styles.featuresTitle, 'text--center')}>
          Why Context-Generic Programming?
        </Heading>
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}
