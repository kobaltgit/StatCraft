'use client';

import React, { useState, useMemo } from 'react';
import { AUDIT_BLOCKS, AuditBlock } from '@/lib/audit-data';
import { SectionShell } from '../section-shell';
import { MathView } from '../math-view';
import { ModalReportIssue } from '../modal-report-issue';
import { CheckCircle2, Search, ChevronDown, ChevronUp, FileCode2, ExternalLink, Flag } from 'lucide-react';

interface OracleBenchmarkProps {
  lang?: 'en' | 'ru';
}

export function OracleBenchmark({ lang = 'en' }: OracleBenchmarkProps) {
  const [searchTerm, setSearchTerm] = useState('');
  const [expandedId, setExpandedId] = useState<string | null>('block-1');
  const [selectedBlockForReport, setSelectedBlockForReport] = useState<string | null>(null);

  const t = {
    en: {
      eyebrow: '02 / MATHEMATICAL AUDIT',
      title: 'The 20-Block SciPy Golden Oracle Matrix',
      lead: 'Every statistical procedure in StatCraft is verified against reference Python/SciPy implementations and classical academic benchmarks. Clean-room implementations without proprietary IBM code.',
      searchPlaceholder: 'Filter by procedure (e.g. T-TEST, LOGISTIC, ANOVA) or keyword...',
      allVerifiedBadge: '20 OF 20 VERIFIED',
      toleranceBadge: 'PRECISION TOLERANCE: Δ ≤ 10⁻⁶',
      referenceText: 'IBM Manual Reference',
      crateFilesText: 'Rust Algo Files',
      formulasHeader: 'Key Formulas & Estimators',
      reportButton: 'Report divergence in this block',
      viewCode: 'View in crates/statcraft_algo',
    },
    ru: {
      eyebrow: '02 / МАТЕМАТИЧЕСКИЙ АУДИТ',
      title: 'Матрица верификации 20 блоков «Золотого оракула» SciPy',
      lead: 'Каждая статистическая процедура StatCraft проверена против референтных реализаций Python/SciPy и классических академических источников. 100% чистая комната без заимствования проприетарного кода.',
      searchPlaceholder: 'Поиск по процедуре (напр. T-TEST, LOGISTIC, ANOVA) или названию...',
      allVerifiedBadge: '20 ИЗ 20 ПРОВЕРЕНО',
      toleranceBadge: 'ДОПУСК ТОЧНОСТИ: Δ ≤ 10⁻⁶',
      referenceText: 'Страницы алгоритмов IBM',
      crateFilesText: 'Файлы алгоритмов Rust',
      formulasHeader: 'Ключевые формулы и оценки',
      reportButton: 'Сообщить о расхождении в этом блоке',
      viewCode: 'Смотреть в crates/statcraft_algo',
    },
  }[lang];

  const filteredBlocks = useMemo(() => {
    if (!searchTerm.trim()) return AUDIT_BLOCKS;
    const term = searchTerm.toLowerCase();
    return AUDIT_BLOCKS.filter(b => {
      const matchName = (lang === 'ru' ? b.titleRu : b.titleEn).toLowerCase().includes(term);
      const matchProc = b.procedures.some(p => p.toLowerCase().includes(term));
      const matchFiles = b.implementationFiles.some(f => f.toLowerCase().includes(term));
      return matchName || matchProc || matchFiles;
    });
  }, [searchTerm, lang]);

  return (
    <SectionShell id="benchmark" eyebrow={t.eyebrow} className="bg-bg">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        {/* Section Header */}
        <div className="mb-10">
          <div className="flex flex-wrap items-center gap-3 mb-4">
            <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded bg-emerald/10 border border-emerald/20 text-emerald text-xs font-mono font-medium">
              <CheckCircle2 className="w-3.5 h-3.5" />
              {t.allVerifiedBadge}
            </span>
            <span className="inline-flex items-center px-2.5 py-1 rounded bg-bg-raised border border-line text-text-muted text-xs font-mono">
              {t.toleranceBadge}
            </span>
          </div>

          <h2 className="text-2xl sm:text-4xl font-bold text-white tracking-tight mb-4">
            {t.title}
          </h2>
          <p className="text-text-muted text-base sm:text-lg max-w-3xl leading-relaxed">
            {t.lead}
          </p>
        </div>

        {/* Filter bar */}
        <div className="relative mb-8">
          <Search className="absolute left-3.5 top-1/2 -translate-y-1/2 w-4 h-4 text-text-muted" />
          <input
            type="text"
            value={searchTerm}
            onChange={e => setSearchTerm(e.target.value)}
            placeholder={t.searchPlaceholder}
            className="w-full pl-10 pr-4 py-2.5 rounded bg-bg-sunken border border-line text-text placeholder:text-text-muted/60 text-sm focus:outline-none focus:border-accent"
          />
          {searchTerm && (
            <button
              onClick={() => setSearchTerm('')}
              className="absolute right-3 top-1/2 -translate-y-1/2 text-xs text-text-muted hover:text-white cursor-pointer"
            >
              Esc
            </button>
          )}
        </div>

        {/* Blocks Accordion / Matrix */}
        <div className="space-y-3">
          {filteredBlocks.map((block) => {
            const isExpanded = expandedId === block.id;
            const title = lang === 'ru' ? block.titleRu : block.titleEn;
            const notes = lang === 'ru' ? block.notesRu : block.notesEn;

            return (
              <div
                key={block.id}
                className={`rounded border transition-colors ${
                  isExpanded ? 'border-line-strong bg-bg-raised/70' : 'border-line bg-bg-raised/30 hover:border-line-strong'
                }`}
              >
                {/* Header row */}
                <button
                  type="button"
                  onClick={() => setExpandedId(isExpanded ? null : block.id)}
                  className="w-full text-left px-4 sm:px-5 py-3.5 sm:py-4 flex items-center justify-between gap-3 sm:gap-4 cursor-pointer"
                >
                  <div className="flex items-center gap-2.5 sm:gap-4 min-w-0 flex-1">
                    <span className="font-mono text-xs sm:text-sm text-text-muted shrink-0 font-semibold">
                      #{String(block.number).padStart(2, '0')}
                    </span>
                    <span className="font-semibold text-white text-sm sm:text-base leading-snug break-words">
                      {title}
                    </span>
                    <div className="hidden sm:flex items-center gap-1.5 shrink-0 flex-wrap">
                      {block.procedures.slice(0, 3).map((p) => (
                        <span
                          key={p}
                          className="px-2 py-0.5 rounded text-[11px] font-mono bg-bg-sunken border border-line text-blue-300"
                        >
                          {p}
                        </span>
                      ))}
                      {block.procedures.length > 3 && (
                        <span className="text-[11px] font-mono text-text-muted">
                          +{block.procedures.length - 3}
                        </span>
                      )}
                    </div>
                  </div>

                  <div className="flex items-center gap-2 sm:gap-3 shrink-0">
                    <span className="hidden md:inline-flex items-center gap-1 text-xs font-mono text-emerald bg-emerald/10 border border-emerald/20 px-2 py-0.5 rounded">
                      <CheckCircle2 className="w-3 h-3" />
                      {block.oraclePrecision}
                    </span>
                    {isExpanded ? (
                      <ChevronUp className="w-4 h-4 text-text-muted shrink-0" />
                    ) : (
                      <ChevronDown className="w-4 h-4 text-text-muted shrink-0" />
                    )}
                  </div>
                </button>

                {/* Expanded Details */}
                {isExpanded && (
                  <div className="px-4 sm:px-5 pb-5 pt-2 border-t border-line/60 space-y-5">
                    {/* Meta info */}
                    <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-xs font-mono">
                      <div className="p-3 rounded bg-bg-sunken border border-line overflow-hidden min-w-0">
                        <div className="text-text-muted mb-1">{t.referenceText}</div>
                        <div className="text-white font-medium break-words">{block.referencePage}</div>
                      </div>

                      <div className="p-3 rounded bg-bg-sunken border border-line md:col-span-2 overflow-hidden min-w-0">
                        <div className="flex flex-wrap items-baseline justify-between gap-x-2 gap-y-1 mb-2">
                          <span className="text-text-muted">{t.crateFilesText}</span>
                          <span className="text-[11px] text-text-muted/60 font-mono break-all">
                            crates/statcraft_algo/src/
                          </span>
                        </div>
                        <div className="flex flex-wrap gap-1.5 text-indigo-300">
                          {block.implementationFiles.map(f => {
                            const isDir = f.endsWith('/');
                            const url = isDir
                              ? `https://github.com/kobaltgit/StatCraft/tree/main/crates/statcraft_algo/src/${f}`
                              : `https://github.com/kobaltgit/StatCraft/blob/main/crates/statcraft_algo/src/${f}`;

                            return (
                              <a
                                key={f}
                                href={url}
                                target="_blank"
                                rel="noopener noreferrer"
                                className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded bg-bg-raised/70 hover:bg-bg-raised border border-line/60 hover:border-indigo-400/40 text-xs font-mono transition-colors group cursor-pointer max-w-full"
                                title={`crates/statcraft_algo/src/${f}`}
                              >
                                <FileCode2 className="w-3.5 h-3.5 text-text-muted group-hover:text-indigo-400 shrink-0" />
                                <span className="group-hover:text-white transition-colors truncate">
                                  {f}
                                </span>
                              </a>
                            );
                          })}
                        </div>
                      </div>
                    </div>

                    {/* Formulas */}
                    {block.formulas && block.formulas.length > 0 && (
                      <div>
                        <div className="text-xs font-mono uppercase tracking-wider text-text-muted mb-3">
                          {t.formulasHeader}
                        </div>
                        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                          {block.formulas.map((f, idx) => (
                            <div
                              key={idx}
                              className="p-3 rounded bg-bg-sunken/80 border border-line flex flex-col justify-between overflow-hidden"
                            >
                              <div className="mb-2">
                                <span className="text-xs font-medium text-amber font-mono break-words">
                                  {f.name}
                                </span>
                                <p className="text-xs text-text-muted mt-0.5 break-words">
                                  {lang === 'ru' ? f.descriptionRu : f.descriptionEn}
                                </p>
                              </div>
                              <div className="py-2 overflow-x-auto text-sm text-center text-white border-t border-line/40 max-w-full">
                                <MathView math={f.latex} block={true} />
                              </div>
                            </div>
                          ))}
                        </div>
                      </div>
                    )}

                    {/* Notes & Actions */}
                    <div className="p-3 rounded bg-bg-sunken/40 border border-line flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 text-xs">
                      <div className="text-text-muted max-w-2xl break-words">
                        <span className="text-text font-medium mr-2">Audit Notes:</span>
                        {notes}
                      </div>

                      <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-2 shrink-0 w-full sm:w-auto">
                        <button
                          type="button"
                          onClick={() => setSelectedBlockForReport(`Block #${block.number}: ${title} (${block.procedures.join(', ')})`)}
                          className="w-full sm:w-auto inline-flex items-center justify-center gap-1.5 px-3 py-1.5 rounded bg-bg text-amber border border-amber/30 hover:bg-amber/10 text-xs font-mono transition-colors cursor-pointer text-center"
                        >
                          <Flag className="w-3 h-3 shrink-0" />
                          <span>{t.reportButton}</span>
                        </button>

                        <a
                          href={`https://github.com/kobaltgit/StatCraft/tree/main/crates/statcraft_algo/src/${block.implementationFiles[0] || ''}`}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="w-full sm:w-auto inline-flex items-center justify-center gap-1 px-3 py-1.5 rounded bg-bg-raised text-text-muted hover:text-white border border-line text-xs font-mono transition-colors text-center shrink-0"
                        >
                          <ExternalLink className="w-3 h-3 shrink-0" />
                          GitHub
                        </a>
                      </div>
                    </div>
                  </div>
                )}
              </div>
            );
          })}
        </div>
      </div>

      {selectedBlockForReport && (
        <ModalReportIssue
          isOpen={true}
          onClose={() => setSelectedBlockForReport(null)}
          lang={lang}
          prefillProcedure={selectedBlockForReport}
        />
      )}
    </SectionShell>
  );
}
