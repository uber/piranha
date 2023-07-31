import React from 'react';
import clsx from 'clsx';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import HomepageFeatures from '@site/src/components/HomepageFeatures';
import AdditionalContent from '@site/src/components/AdditionalContent';

import styles from './index.module.css';
import '../css/custom.css';

function HomepageHeader() {
    const {siteConfig} = useDocusaurusContext();
    return (
        <header className={clsx('hero hero--primary', styles.heroBanner)}>
            <div className="container" style={{textAlign: "left"}}>
                <h1 className="hero__title">{siteConfig.title}</h1>
                <div className="row">
                    <div className="col col--6">
                        <p className="hero__subtitle">{siteConfig.tagline}</p>
                    </div>
                </div>
                <div className={styles.buttons} style={{justifyContent: "flex-start"}}>
                    <Link
                        className="button button--secondary button--lg"
                        to="/docs/reference/getting-started/install">
                        Getting Started
                    </Link>
                </div>
            </div>
        </header>
    );
}

export default function Home() {
    const {siteConfig} = useDocusaurusContext();
    return (
        <Layout
            title={`${siteConfig.title}`}
            description="Piranha is a tool for refactoring and transforming code at scale.">
            <HomepageHeader />
            <main>
                <HomepageFeatures />
                <AdditionalContent />
            </main>
        </Layout>
    );
}
