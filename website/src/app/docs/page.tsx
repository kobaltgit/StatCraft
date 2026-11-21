import { DOC_SECTIONS } from '@/lib/docs-data';
import { DocPageView } from '@/components/docs/doc-page-view';
import { notFound } from 'next/navigation';

export default function DocsIndexPage() {
  const section = DOC_SECTIONS[0];
  if (!section) return notFound();

  return <DocPageView section={section} lang="en" />;
}
