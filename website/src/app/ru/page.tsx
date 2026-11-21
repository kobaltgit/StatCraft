import React from 'react';
import { SiteNav } from '@/components/site-nav';
import { Hero } from '@/components/landing/hero';
import { OracleBenchmark } from '@/components/landing/oracle-benchmark';
import { AgentLab } from '@/components/landing/agent-lab';
import { ArchitectureGuarantee } from '@/components/landing/architecture-guarantee';
import { CallForResearchers } from '@/components/landing/call-for-researchers';
import { SiteFooter } from '@/components/site-footer';
import { FixedCornerNav } from '@/components/fixed-corner-nav';

export default function RussianHomePage() {
  return (
    <div className="relative min-h-screen flex flex-col bg-bg">
      <SiteNav lang="ru" />
      <main className="flex-1">
        <Hero lang="ru" />
        <OracleBenchmark lang="ru" />
        <AgentLab lang="ru" />
        <ArchitectureGuarantee lang="ru" />
        <CallForResearchers lang="ru" />
      </main>
      <SiteFooter lang="ru" />
      <FixedCornerNav lang="ru" />
    </div>
  );
}
