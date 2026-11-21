import { DOC_SECTIONS } from '@/lib/docs-data';
import { DocPageView } from '@/components/docs/doc-page-view';
import { notFound } from 'next/navigation';
import type { Metadata } from 'next';

interface PageProps {
  params: Promise<{ slug: string }>;
}

export async function generateStaticParams() {
  return DOC_SECTIONS.map((section) => ({
    slug: section.slug,
  }));
}

export async function generateMetadata({ params }: PageProps): Promise<Metadata> {
  const { slug } = await params;
  const section = DOC_SECTIONS.find((s) => s.slug === slug);
  if (!section) return { title: 'Не найдено // StatCraft Вики' };

  return {
    title: `${section.titleRu} // Вики StatCraft`,
    description: section.descriptionRu,
  };
}

export default async function RussianDocDetailPage({ params }: PageProps) {
  const { slug } = await params;
  const section = DOC_SECTIONS.find((s) => s.slug === slug);

  if (!section) {
    notFound();
  }

  return <DocPageView section={section} lang="ru" />;
}
