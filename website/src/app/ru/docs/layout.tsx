import React from 'react';
import { DocsShell } from '@/components/docs/docs-shell';

export default function RussianDocsLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return <DocsShell lang="ru">{children}</DocsShell>;
}
