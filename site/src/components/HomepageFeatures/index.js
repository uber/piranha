import React from 'react';
import clsx from 'clsx';
import styles from './styles.module.css';

const FeatureList = [
  {
    title: 'Structural find / replace',
    // Svg: require('@site/static/img/undraw_docusaurus_tree.svg').default,
    description: (
      <>
          Find and replace code in <b>any declarative matching language</b> (regex, tree-sitter queries).
          Specify extra matchers in our filter language to achieve finely-grained rewrites.
      </>
    ),
  },
  {
    title: 'Chain rewrite rules',
    // Svg: require('@site/static/img/undraw_docusaurus_react.svg').default,
    description: (
      <>
          Cascade program transformations by chaining your rewrite rules with our graph language.
          Piranha will automatically refactor your code with your chaining strategy.
      </>
    ),
  },
];

// <!--<Svg className={styles.featureSvg} role="img" />-->

function Feature({Svg, title, description}) {
  return (
    <div className={clsx('col col--6')}>
      <div className="">
      </div>
      <div className="padding-horiz--md">
        <h3>{title}</h3>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures() {
    return (
        <section className={styles.features}>
            <div className="container">
                <div className={clsx('row', styles.centeredRow)}>
                    {FeatureList.map((props, idx) => (
                        <Feature key={idx} {...props} />
                    ))}
                </div>
            </div>
        </section>
    );
}
