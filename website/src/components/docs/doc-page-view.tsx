'use client';

import React from 'react';
import Link from 'next/link';
import { DocSection, DOC_SECTIONS } from '@/lib/docs-data';
import { MarkdownRenderer } from './markdown-renderer';
import { SpssSyntaxBlock } from './spss-syntax-block';
import { DocsToc } from './docs-toc';
import { CheckCircle2, BookOpen, ArrowLeft, ArrowRight, ShieldCheck } from 'lucide-react';

interface DocPageViewProps {
  section: DocSection;
  lang?: 'en' | 'ru';
}

export function DocPageView({ section, lang = 'en' }: DocPageViewProps) {
  const currentIndex = DOC_SECTIONS.findIndex(s => s.slug === section.slug);
  const prevSection = currentIndex > 0 ? DOC_SECTIONS[currentIndex - 1] : null;
  const nextSection = currentIndex < DOC_SECTIONS.length - 1 ? DOC_SECTIONS[currentIndex + 1] : null;

  const basePath = lang === 'ru' ? '/ru/docs' : '/docs';
  const title = lang === 'ru' ? section.titleRu : section.titleEn;
  const description = lang === 'ru' ? section.descriptionRu : section.descriptionEn;
  const categoryTitle = lang === 'ru' ? section.categoryTitleRu : section.categoryTitleEn;
  const content = lang === 'ru' ? section.contentRu : section.contentEn;

  return (
    <div className="flex flex-col xl:flex-row min-h-full">
      {/* Center Reading Column */}
      <main className="flex-1 min-w-0 px-6 sm:px-10 py-10 max-w-4xl">
        {/* Breadcrumb */}
        <div className="flex items-center gap-2 text-xs font-mono text-text-muted mb-4">
          <Link href={basePath} className="hover:text-white transition-colors">
            {lang === 'ru' ? 'Вики' : 'Wiki'}
          </Link>
          <span>/</span>
          <span className="text-white/80">{categoryTitle}</span>
        </div>

        {/* Title & Badges */}
        <h1 className="text-2xl sm:text-4xl font-bold text-white tracking-tight mb-4">
          {title}
        </h1>

        <p className="text-base sm:text-lg text-text-muted leading-relaxed mb-6">
          {description}
        </p>

        {/* Audit & Specification Badges */}
        <div className="flex flex-wrap items-center gap-3 mb-8 pb-6 border-b border-line">
          {section.oracleStatus && (
            <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded bg-emerald/10 border border-emerald/20 text-emerald text-xs font-mono">
              <CheckCircle2 className="w-3.5 h-3.5" />
              <span>{section.oracleStatus}</span>
            </span>
          )}

          {section.spssPage && (
            <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded bg-bg-raised border border-line text-text-muted text-xs font-mono">
              <BookOpen className="w-3.5 h-3.5 text-amber" />
              <span>{section.spssPage}</span>
            </span>
          )}
        </div>

        {/* Render Markdown Content with KaTeX */}
        <MarkdownRenderer content={content} toc={section.toc} />

        {/* Optional SPSS Syntax Block */}
        {section.syntaxExample && (
          <div className="mt-8 pt-6 border-t border-line">
            <h4 className="text-sm font-bold text-white font-mono uppercase tracking-wider mb-2">
              {lang === 'ru' ? 'Пример классического синтаксиса SPSS' : 'Canonical SPSS Syntax Example'}
            </h4>
            <SpssSyntaxBlock syntax={section.syntaxExample} />
          </div>
        )}

        {/* Pagination Prev / Next */}
        <div className="mt-12 pt-8 border-t border-line flex flex-col sm:flex-row items-center justify-between gap-4">
          {prevSection ? (
            <Link
              href={`${basePath}/${prevSection.slug}`}
              className="w-full sm:w-auto p-4 rounded border border-line bg-bg-raised/40 hover:bg-bg-raised text-left flex items-center gap-3 transition-colors"
            >
              <ArrowLeft className="w-4 h-4 text-text-muted shrink-0" />
              <div>
                <div className="text-[11px] font-mono text-text-muted uppercase">
                  {lang === 'ru' ? 'Предыдущая глава' : 'Previous Section'}
                </div>
                <div className="text-xs font-medium text-white truncate max-w-[200px]">
                  {lang === 'ru' ? prevSection.titleRu : prevSection.titleEn}
                </div>
              </div>
            </Link>
          ) : <div />}

          {nextSection && (
            <Link
              href={`${basePath}/${nextSection.slug}`}
              className="w-full sm:w-auto p-4 rounded border border-line bg-bg-raised/40 hover:bg-bg-raised text-right flex items-center justify-end gap-3 transition-colors ml-auto"
            >
              <div>
                <div className="text-[11px] font-mono text-text-muted uppercase">
                  {lang === 'ru' ? 'Следующая глава' : 'Next Section'}
                </div>
                <div className="text-xs font-medium text-white truncate max-w-[200px]">
                  {lang === 'ru' ? nextSection.titleRu : nextSection.titleEn}
                </div>
              </div>
              <ArrowRight className="w-4 h-4 text-text-muted shrink-0" />
            </Link>
          )}
        </div>
      </main>

      {/* Right Column: TOC & Report Trigger (Sticky on XL) */}
      <div className="hidden xl:block w-72 shrink-0 border-l border-line p-6 sticky top-14 h-[calc(100vh-3.5rem)] overflow-y-auto">
        <DocsToc section={section} lang={lang} />
      </div>
    </div>
  );
}
