'use client';

import React, { useState } from 'react';
import { DocSection } from '@/lib/docs-data';
import { ModalReportIssue } from '../modal-report-issue';
import { List, Flag, ExternalLink } from 'lucide-react';

interface DocsTocProps {
  section: DocSection;
  lang?: 'en' | 'ru';
  className?: string;
}

export function DocsToc({ section, lang = 'en', className = '' }: DocsTocProps) {
  const [reportModalOpen, setReportModalOpen] = useState(false);

  const t = {
    en: {
      onThisPage: 'ON THIS PAGE',
      reportIssue: 'Report discrepancy here',
      githubSource: 'View crates/statcraft_algo',
    },
    ru: {
      onThisPage: 'НА ЭТОЙ СТРАНИЦЕ',
      reportIssue: 'Сообщить о расхождении',
      githubSource: 'Код crates/statcraft_algo',
    },
  }[lang];

  return (
    <div className={`space-y-6 ${className}`}>
      {/* TOC list */}
      {section.toc && section.toc.length > 0 && (
        <div>
          <div className="flex items-center gap-2 text-xs font-mono font-medium text-text-muted uppercase tracking-wider mb-3">
            <List className="w-3.5 h-3.5" />
            <span>{t.onThisPage}</span>
          </div>

          <nav className="space-y-2 border-l border-line pl-3">
            {section.toc.map((item) => {
              const title = lang === 'ru' ? item.titleRu : item.titleEn;
              // anchor slug matches heading slug in markdown-renderer
              const slug = item.titleEn
                .toLowerCase()
                .replace(/[^\w\s-]/g, '')
                .replace(/\s+/g, '-');

              return (
                <a
                  key={item.id}
                  href={`#${slug}`}
                  className="block text-xs text-text-muted hover:text-white transition-colors leading-relaxed"
                >
                  {title}
                </a>
              );
            })}
          </nav>
        </div>
      )}

      {/* Quick Action Buttons */}
      <div className="pt-4 border-t border-line space-y-2">
        <button
          type="button"
          onClick={() => setReportModalOpen(true)}
          className="w-full inline-flex items-center gap-2 px-3 py-2 rounded bg-amber/10 border border-amber/20 text-amber hover:bg-amber/20 text-xs font-mono transition-colors"
        >
          <Flag className="w-3.5 h-3.5" />
          <span>{t.reportIssue}</span>
        </button>

        <a
          href="https://github.com/kobaltgit/StatCraft"
          target="_blank"
          rel="noopener noreferrer"
          className="w-full inline-flex items-center gap-2 px-3 py-2 rounded bg-bg-raised border border-line text-text-muted hover:text-white text-xs font-mono transition-colors"
        >
          <ExternalLink className="w-3.5 h-3.5" />
          <span>{t.githubSource}</span>
        </a>
      </div>

      <ModalReportIssue
        isOpen={reportModalOpen}
        onClose={() => setReportModalOpen(false)}
        lang={lang}
        prefillProcedure={`${section.titleEn} (${section.slug})`}
      />
    </div>
  );
}
