/**
 * (c) Meta Platforms, Inc. and affiliates. Confidential and proprietary.
 */

import React from 'react';
import clsx from 'clsx';
import styles from './HomepageFeatures.module.css';

/**
 * Use an empty element until we have imagery to complement the text. Note that
 * Buck2 does not use SVGs here, but emoji:
 * https://github.com/facebook/buck2/blob/5b0aa923ea621a02331612f7e557d5c946c44561/website/src/components/HomepageFeatures.js#L16
 */
function EmptyElement() {
  return <></>;
}

const FeatureList = [
  {
    title: 'Simple',
    Svg: EmptyElement,
    description: (
      <>
        DotSlash enables you to replace a set of platform-specific, heavyweight
        executables with an equivalent small, easy-to-read text file.
      </>
    ),
  },
  {
    title: 'No Overhead',
    Svg: EmptyElement,
    description: (
      <>
        DotSlash is written in Rust so it can run your executables quickly
        and transparently.
      </>
    ),
  },
  {
    title: 'Painless Automation',
    Svg: EmptyElement,
    description: (
      <>
        We provide tools for <a href="./docs/github/">generating DotSlash
          files</a> for GitHub releases.
      </>
    ),
  },
];

function Feature({ Svg, title, description }) {
  return (
    <div className={clsx('col col--4')}>
      <div className="text--center">
        <Svg className={styles.featureSvg} alt={title} />
      </div>
      <div className="text--center padding-horiz--md">
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
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}
