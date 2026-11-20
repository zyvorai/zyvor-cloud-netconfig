import type {ReactNode} from 'react';
import clsx from 'clsx';
import Link from '@docusaurus/Link';
import Layout from '@theme/Layout';
import Heading from '@theme/Heading';
import FeatureHighlights from '@site/src/components/FeatureHighlights';
import Reveal from '@site/src/components/Reveal';

import styles from './index.module.css';

function HomepageHeader() {
  return (
    <header className={clsx('hero hero--primary', styles.heroBanner)}>
      <div className="container">
        <div className={clsx(styles.heroGridSingle, 'text--center')}>
          <Heading as="h1" className="hero__title">
            Automatic network
            <br />
            configuration for cloud VMs.
          </Heading>
          <p className="hero__subtitle">
            Automatic network configuration for cloud instances using
            provider metadata — Azure, AWS, GCP, and others. Handles
            secondary IPs, routing tables, and policy-based routing on
            multi-interface VMs, unprivileged with just{' '}
            <code>CAP_NET_ADMIN</code>.
          </p>
          <div className={styles.buttons}>
            <Link
              className="button button--secondary button--lg"
              href="https://github.com/hypersdk/cloud-netconfig#installation">
              Get Started
            </Link>
            <Link
              className="button button--outline button--lg button--secondary"
              to="https://github.com/hypersdk/cloud-netconfig">
              View on GitHub
            </Link>
          </div>
        </div>
      </div>
    </header>
  );
}

function ProblemStatement() {
  return (
    <section className={styles.problem}>
      <div className="container">
        <Reveal className="row">
          <div className="col col--8 col--offset-2 text--center">
            <Heading as="h2" className={styles.sectionHeading}>
              Why cloud-netconfig
            </Heading>
            <p>
              Multi-interface cloud VMs need their secondary NICs, routing
              tables, and policy routes configured correctly, and that
              configuration comes from provider-specific instance metadata
              — different shapes on Azure, AWS, and GCP, and different
              again on other clouds. Getting it wrong means unreachable
              interfaces or asymmetric routing that's painful to debug.
            </p>
            <p>
              cloud-netconfig runs as an unprivileged daemon
              (<code>cloud-netconfigd</code>) that watches provider
              metadata and reconfigures interfaces and policy routing
              automatically over netlink as instances change — no manual
              per-cloud network scripts to maintain.
            </p>
          </div>
        </Reveal>
      </div>
    </section>
  );
}

function TrustBand() {
  return (
    <section className={styles.trust}>
      <div className="container">
        <Reveal className={styles.trustGrid}>
          <div>
            <Heading as="h3" className={styles.sectionHeading}>
              Source-available, Community Edition
            </Heading>
            <p>
              Daemon under the Zyvor Production License, maintained by Zyvor AI Labs.
              Runs unprivileged with <code>CAP_NET_ADMIN</code> rather than
              full root. Enterprise adds fleet automation, SLAs, and
              HyperSDK migration/operations integration on the same
              codebase.
            </p>
            <Link to="/docs/enterprise">See Community vs. Enterprise →</Link>
          </div>
          <div className={styles.trustBadges}>
            <img
              src="https://github.com/hypersdk/cloud-netconfig/actions/workflows/ci.yml/badge.svg"
              alt="CI status"
            />
            <img
              src="https://img.shields.io/badge/license-Zyvor%20Production%201.0-blue.svg"
              alt="Zyvor Production License v1.0"
            />
          </div>
        </Reveal>
      </div>
    </section>
  );
}

function EnterpriseCTA() {
  return (
    <section className={styles.enterprise}>
      <div className="container text--center">
        <Reveal>
          <Heading as="h2" className={styles.sectionHeading}>
            Need supported multi-cloud rollouts?
          </Heading>
          <p className={styles.enterpriseCopy}>
            Community Edition is free for non-production use; production use
            needs a commercial license. Zyvor Enterprise
            adds SLAs, professional services, and HyperSDK migration
            integration for production multi-cloud fleets.
          </p>
          <Link
            className="button button--primary button--lg"
            href="mailto:sales@zyvor.dev">
            Contact sales@zyvor.dev
          </Link>
        </Reveal>
      </div>
    </section>
  );
}

export default function Home(): ReactNode {
  return (
    <Layout
      title="cloud-netconfig — automatic cloud network configuration"
      description="Automatic network configuration for cloud instances using provider metadata (Azure, AWS, GCP, and others). Handles secondary IPs, routing tables, and policy-based routing on multi-interface VMs.">
      <HomepageHeader />
      <main>
        <ProblemStatement />
        <Reveal>
          <FeatureHighlights />
        </Reveal>
        <TrustBand />
        <EnterpriseCTA />
      </main>
    </Layout>
  );
}
