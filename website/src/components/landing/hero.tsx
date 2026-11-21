'use client';

import React, { useState } from 'react';
import Link from 'next/link';
import { ArrowRight, AlertTriangle, ShieldCheck, Cpu, Database, Binary, MessageSquarePlus, Coffee } from 'lucide-react';
import { ModalReportIssue } from '../modal-report-issue';
import { DownloadButton } from '../download-button';
import { useRelease } from '@/lib/use-release';

interface HeroProps {
  lang?: 'en' | 'ru';
}

export function Hero({ lang = 'en' }: HeroProps) {
  const [reportModalOpen, setReportModalOpen] = useState(false);
  const { version } = useRelease();

  const t = {
    en: {
      badge: `ACADEMIC EXPERIMENT // BUILD ${version}`,
      titleP1: 'Open-Science Statistical Computing.',
      titleP2: 'Verified to 10⁻⁶ Precision.',
      lead: 'An experimental, 100% Safe Rust alternative for statistical data analysis. Clean-room mathematical engine built on Apache Arrow and Polars with native Model Context Protocol (MCP) integration for AI-guided empirical research.',
      disclaimer: 'Academic Notice: StatCraft is an independent open-science software project. It is not affiliated with or certified by IBM SPSS Statistics. Do not use for clinical diagnostic decisions or legal compliance without independent statistical replication.',
      btnDocs: 'Documentation & Wiki',
      btnAudit: '20-Block Audit Matrix',
      btnGithub: 'Source on GitHub',
      btnReport: 'Report Discrepancy',
      stat1Val: '20 / 20',
      stat1Lbl: 'Audit Blocks Verified',
      stat1Desc: 'Independent validation against SciPy Golden Oracle (Δ < 10⁻⁶)',
      stat2Val: '0 Panics',
      stat2Lbl: '100% Safe Rust Architecture',
      stat2Desc: 'No unwrap, no unsafe, typed Result<T, E> error propagation',
      stat3Val: 'Apache Arrow',
      stat3Lbl: 'Columnar Data Engine',
      stat3Desc: 'Zero-copy memory layouts, Polars vectorization, SPSS .sav codec',
      stat4Val: 'MCP & IPC',
      stat4Lbl: 'Autonomous Agent Bridge',
      stat4Desc: 'Real-time JSON-RPC socket 18492 for Antigravity, Claude, and local LLMs',
    },
    ru: {
      badge: `АКАДЕМИЧЕСКИЙ ЭКСПЕРИМЕНТ // СБОРКА ${version}`,
      titleP1: 'Статистические вычисления открытой науки.',
      titleP2: 'Проверено с точностью до 10⁻⁶.',
      lead: 'Экспериментальная альтернатива классическим статистическим пакетам на 100% безопасном Rust. Чистая математическая реализация на базе Apache Arrow и Polars со встроенным протоколом MCP для совместных исследований с ИИ.',
      disclaimer: 'Уведомление об исследовательской природе: StatCraft — независимый проект открытой науки. Программа не аффилирована с IBM SPSS Statistics. Не предназначена для клинических медицинских диагнозов и судебной экспертизы без независимого воспроизведения расчетов.',
      btnDocs: 'Документация и Вики',
      btnAudit: 'Матрица аудита 20 блоков',
      btnGithub: 'Исходный код GitHub',
      btnReport: 'Сообщить о расхождении',
      stat1Val: '20 из 20',
      stat1Lbl: 'Блоков аудита проверено',
      stat1Desc: 'Сверка с «Золотым оракулом» SciPy (допуск Δ < 10⁻⁶)',
      stat2Val: '0 паник',
      stat2Lbl: '100% Safe Rust архитектура',
      stat2Desc: 'Полный отказ от unwrap и unsafe, строгая типизация ошибок Result<T, E>',
      stat3Val: 'Apache Arrow',
      stat3Lbl: 'Колоночный движок памяти',
      stat3Desc: 'Zero-copy структуры, векторные расчеты Polars, открытый парсер .sav',
      stat4Val: 'MCP и сокет',
      stat4Lbl: 'Мост для ИИ-агентов',
      stat4Desc: 'IPC-сокет 18492 и Model Context Protocol для Antigravity, Claude и локальных LLM',
    },
  }[lang];

  return (
    <section id="manifest" className="relative pt-12 pb-20 border-b border-line bg-gradient-to-b from-bg to-bg-sunken scroll-mt-14">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        {/* Academic status pill */}
        <div className="inline-flex items-center gap-2 px-3 py-1 rounded border border-amber/30 bg-amber/10 text-amber text-xs font-mono font-medium tracking-wider mb-6">
          <span className="w-1.5 h-1.5 rounded-full bg-amber animate-pulse" />
          {t.badge}
        </div>

        {/* Main Title */}
        <h1 className="text-3xl sm:text-5xl lg:text-6xl font-bold tracking-tight text-white mb-6">
          <span>{t.titleP1}</span>{' '}
          <span className="font-serif italic font-normal text-transparent bg-clip-text bg-gradient-to-r from-blue-400 via-indigo-300 to-emerald-400">
            {t.titleP2}
          </span>
        </h1>

        {/* Lead paragraph */}
        <p className="max-w-3xl text-lg sm:text-xl text-text-muted leading-relaxed mb-8">
          {t.lead}
        </p>

        {/* Critical Academic Disclaimer */}
        <div className="mb-10 p-4 rounded border border-amber/20 bg-amber/5 text-amber-200/90 text-xs sm:text-sm leading-relaxed flex items-start gap-3">
          <AlertTriangle className="w-5 h-5 text-amber shrink-0 mt-0.5" />
          <div className="break-words">{t.disclaimer}</div>
        </div>

        {/* Primary CTA Buttons */}
        <div className="flex flex-col sm:flex-row sm:flex-wrap items-stretch sm:items-center gap-3 sm:gap-4 mb-16">
          {/* Direct Binary Download for Auto-Detected OS */}
          <DownloadButton lang={lang} variant="hero" />

          <Link
            href={lang === 'ru' ? '/ru/docs' : '/docs'}
            className="w-full sm:w-auto inline-flex items-center justify-center gap-2 px-5 py-3 rounded bg-bg-raised text-text font-medium hover:bg-bg-raised/80 border border-line text-sm transition-colors text-center"
          >
            <span>{t.btnDocs}</span>
            <ArrowRight className="w-4 h-4 shrink-0" />
          </Link>

          <a
            href="#benchmark"
            className="w-full sm:w-auto inline-flex items-center justify-center gap-2 px-5 py-3 rounded bg-bg-raised text-text font-medium hover:bg-bg-raised/80 border border-line text-sm transition-colors text-center"
          >
            <Binary className="w-4 h-4 text-emerald shrink-0" />
            <span>{t.btnAudit}</span>
          </a>

          <a
            href="https://github.com/kobaltgit/StatCraft"
            target="_blank"
            rel="noopener noreferrer"
            className="w-full sm:w-auto inline-flex items-center justify-center gap-2 px-5 py-3 rounded bg-bg-raised text-text-muted font-medium hover:text-white border border-line text-sm transition-colors text-center"
          >
            <svg className="w-4 h-4 fill-current shrink-0" viewBox="0 0 24 24">
              <path fillRule="evenodd" clipRule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.53 1.032 1.53 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" />
            </svg>
            <span>{t.btnGithub}</span>
          </a>

          {/* Support author on Lava.top */}
          <a
            href="https://app.lava.top/kobaltgit"
            target="_blank"
            rel="noopener noreferrer"
            className="w-full sm:w-auto group inline-flex items-center justify-center gap-2 px-5 py-3 rounded border border-amber/40 bg-amber/10 text-amber hover:bg-amber hover:text-black font-medium text-sm transition-all shadow-sm text-center"
            title="Buy me a coffee on Lava.top"
          >
            <Coffee className="w-4 h-4 shrink-0 transition-colors" />
            <span>{lang === 'ru' ? 'Поддержать (Lava.top)' : 'Buy me Coffee'}</span>
          </a>

          <button
            type="button"
            onClick={() => setReportModalOpen(true)}
            className="w-full sm:w-auto inline-flex items-center justify-center gap-2 px-4 py-3 rounded border border-line/60 sm:border-transparent bg-bg-raised/40 sm:bg-transparent text-text-muted hover:text-amber text-sm font-mono transition-colors cursor-pointer text-center"
          >
            <MessageSquarePlus className="w-4 h-4 text-amber shrink-0" />
            <span>{t.btnReport}</span>
          </button>
        </div>

        {/* 4 Feature / Guarantee Badges */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
          <div className="p-5 rounded border border-line bg-bg-raised/40 hover:border-emerald/40 transition-colors">
            <div className="flex items-center justify-between mb-3">
              <span className="text-2xl font-bold font-mono text-emerald">{t.stat1Val}</span>
              <ShieldCheck className="w-5 h-5 text-emerald" />
            </div>
            <div className="font-medium text-white text-sm mb-1">{t.stat1Lbl}</div>
            <div className="text-xs text-text-muted leading-relaxed">{t.stat1Desc}</div>
          </div>

          <div className="p-5 rounded border border-line bg-bg-raised/40 hover:border-blue-400/40 transition-colors">
            <div className="flex items-center justify-between mb-3">
              <span className="text-2xl font-bold font-mono text-blue-400">{t.stat2Val}</span>
              <Cpu className="w-5 h-5 text-blue-400" />
            </div>
            <div className="font-medium text-white text-sm mb-1">{t.stat2Lbl}</div>
            <div className="text-xs text-text-muted leading-relaxed">{t.stat2Desc}</div>
          </div>

          <div className="p-5 rounded border border-line bg-bg-raised/40 hover:border-indigo-400/40 transition-colors">
            <div className="flex items-center justify-between mb-3">
              <span className="text-2xl font-bold font-mono text-indigo-400">{t.stat3Val}</span>
              <Database className="w-5 h-5 text-indigo-400" />
            </div>
            <div className="font-medium text-white text-sm mb-1">{t.stat3Lbl}</div>
            <div className="text-xs text-text-muted leading-relaxed">{t.stat3Desc}</div>
          </div>

          <div className="p-5 rounded border border-line bg-bg-raised/40 hover:border-amber/40 transition-colors">
            <div className="flex items-center justify-between mb-3">
              <span className="text-2xl font-bold font-mono text-amber">{t.stat4Val}</span>
              <Binary className="w-5 h-5 text-amber" />
            </div>
            <div className="font-medium text-white text-sm mb-1">{t.stat4Lbl}</div>
            <div className="text-xs text-text-muted leading-relaxed">{t.stat4Desc}</div>
          </div>
        </div>
      </div>

      <ModalReportIssue
        isOpen={reportModalOpen}
        onClose={() => setReportModalOpen(false)}
        lang={lang}
      />
    </section>
  );
}
