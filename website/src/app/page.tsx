import React from 'react';
import { SiteNav } from '@/components/site-nav';
import { Hero } from '@/components/landing/hero';
import { OracleBenchmark } from '@/components/landing/oracle-benchmark';
import { AgentLab } from '@/components/landing/agent-lab';
import { ArchitectureGuarantee } from '@/components/landing/architecture-guarantee';
import { CallForResearchers } from '@/components/landing/call-for-researchers';
import { SiteFooter } from '@/components/site-footer';
import { FixedCornerNav } from '@/components/fixed-corner-nav';

export default function HomePage() {
  return (
    <div className="relative min-h-screen flex flex-col bg-bg">
      <SiteNav lang="en" />
      <main className="flex-1">
        <Hero lang="en" />
        <OracleBenchmark lang="en" />
        <AgentLab lang="en" />
        <ArchitectureGuarantee lang="en" />
        <CallForResearchers lang="en" />
      </main>
      <SiteFooter lang="en" />
      <FixedCornerNav lang="en" />
    </div>
  );
}
