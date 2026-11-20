// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: LicenseRef-Zyvor-Production-1.0

import type {ReactNode} from 'react';
import Link from '@docusaurus/Link';
import Heading from '@theme/Heading';
import styles from './styles.module.css';

type FeatureItem = {
  title: string;
  description: ReactNode;
  to: string;
};

const FeatureList: FeatureItem[] = [
  {
    title: 'Multi-cloud metadata clients',
    description:
      'Built-in clients for Azure, AWS EC2, GCP, and more — reads each provider\'s own instance metadata shape instead of one-size-fits-all guessing.',
    to: '/docs/user/getting-started',
  },
  {
    title: 'Event-driven reconfiguration',
    description:
      'Watches for interface and metadata changes and reconfigures over netlink automatically, instead of a cron job re-running static scripts.',
    to: '/docs/user/configuration',
  },
  {
    title: 'Policy-based routing',
    description:
      'Handles multi-homed hosts correctly — secondary IPs and per-interface routing tables so traffic goes out the interface it arrived on.',
    to: '/docs/user/configuration',
  },
  {
    title: 'Local HTTP API',
    description:
      'Query instance metadata locally without going back to the cloud provider\'s metadata service from every script that needs it.',
    to: '/docs/user/using-the-dashboard',
  },
  {
    title: 'Unprivileged by design',
    description:
      'Runs as a dedicated cloud-network user with just CAP_NET_ADMIN, not full root — a smaller blast radius if the daemon is ever compromised.',
    to: '/docs/user/admin-basics',
  },
  {
    title: 'cnctl CLI',
    description:
      'Inspect live status and interfaces from the command line — cnctl status system, cnctl show interfaces — for scripting and troubleshooting.',
    to: '/docs/user/workflows',
  },
];

function Feature({title, description, to}: FeatureItem) {
  return (
    <div className="col col--4">
      <Link to={to} className={styles.card}>
        <Heading as="h3">{title}</Heading>
        <p>{description}</p>
      </Link>
    </div>
  );
}

export default function FeatureHighlights(): ReactNode {
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
