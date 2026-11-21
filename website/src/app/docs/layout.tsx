import React from 'react';
import { DocsShell } from '@/components/docs/docs-shell';

export default function EnglishDocsLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return <DocsShell lang="en">{children}</DocsShell>;
}
