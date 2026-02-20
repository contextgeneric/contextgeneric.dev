import type {ReactNode} from 'react';
import clsx from 'clsx';
import Heading from '@theme/Heading';
import styles from './styles.module.css';

type FeatureItem = {
  title: string;
  Svg: React.ComponentType<React.ComponentProps<'svg'>>;
  description: ReactNode;
};

const FeatureList: FeatureItem[] = [
  {
    title: 'Modular Component System',
    Svg: require('@site/static/img/undraw_docusaurus_mountain.svg').default,
    description: (
      <>
        Decouple interface definitions from implementations using provider traits.
        Write multiple context-generic providers that can coexist without conflicts.
      </>
    ),
  },
  {
    title: 'Highly Expressive Macros',
    Svg: require('@site/static/img/undraw_docusaurus_tree.svg').default,
    description: (
      <>
        Write abstract programs generic over contexts without managing long lists
        of generic parameters. Achieve expressiveness rivaling dynamic languages.
      </>
    ),
  },
  {
    title: 'Type-Safe Composition',
    Svg: require('@site/static/img/undraw_docusaurus_react.svg').default,
    description: (
      <>
        All component wiring is verified at compile time. Missing dependencies are
        caught early, with no runtime errors possible from CGP constructs.
      </>
    ),
  },
  {
    title: 'No-Std Friendly',
    Svg: require('@site/static/img/undraw_docusaurus_mountain.svg').default,
    description: (
      <>
        Build fully abstract programs without concrete dependencies. Deploy to
        embedded systems, kernels, WebAssembly, or symbolic execution platforms.
      </>
    ),
  },
  {
    title: 'Zero-Cost Abstraction',
    Svg: require('@site/static/img/undraw_docusaurus_tree.svg').default,
    description: (
      <>
        All CGP operations happen at compile time using Rust's type system.
        No runtime overhead, upholding Rust's zero-cost abstraction guarantee.
      </>
    ),
  },
  {
    title: 'Bypass Coherence Rules',
    Svg: require('@site/static/img/undraw_docusaurus_react.svg').default,
    description: (
      <>
        Enable overlapping and orphan trait implementations without restriction.
        Define implementations for types you don't own and traits you don't control.
      </>
    ),
  },
];

function Feature({title, Svg, description}: FeatureItem) {
  return (
    <div className={clsx('col col--4')}>
      <div className="text--center">
        <Svg className={styles.featureSvg} role="img" />
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
