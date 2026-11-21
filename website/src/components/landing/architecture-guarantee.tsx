'use client';

import React from 'react';
import { SectionShell } from '../section-shell';
import { Layers, ShieldAlert, Cpu, Sparkles, Binary, CheckCircle2 } from 'lucide-react';

interface ArchitectureGuaranteeProps {
  lang?: 'en' | 'ru';
}

export function ArchitectureGuarantee({ lang = 'en' }: ArchitectureGuaranteeProps) {
  const t = {
    en: {
      eyebrow: '04 / ARCHITECTURAL INTEGRITY',
      title: 'Built on 100% Safe Rust & Apache Arrow',
      lead: 'Statistical software must never crash mid-calculation or produce silent NaN errors. StatCraft enforces strict memory safety, strict layered dependencies, and a zero-panic contract.',
      layerHeading: 'Layered Hierarchy (L0 → L6)',
      layerDesc: 'Enforced at compile-time by cargo xtask layers. No cyclic dependencies, no UI logic leaking into math.',
      pillar1Title: 'Zero-Panic Contract',
      pillar1Desc: 'All non-test code strictly forbids unwrap(), expect(), panic!(), and unsafe. Every potential error (singular matrix, division by zero, insufficient sample size) is propagated as typed Result<T, StatError>.',
      pillar2Title: 'Clean-Room Implementation',
      pillar2Desc: 'Developed without a single line of proprietary IBM SPSS code or assets. Algorithms are derived strictly from published academic papers, ISO standards, and GNU PSPP open-source specifications.',
      pillar3Title: 'Arrow Columnar Memory',
      pillar3Desc: 'Datasets live in contiguous Apache Arrow arrays. Numerical transformations and filtering execute via SIMD-accelerated Polars engines with zero GC pauses.',
      pillar4Title: 'Vector-First Design System',
      pillar4Desc: 'Native desktop GUI built in egui/eframe. Custom DialogKit with strict Design Guard checks, sub-millisecond render cycles, and 0 Unicode tofu glyphs.',
    },
    ru: {
      eyebrow: '04 / НАДЕЖНОСТЬ АРХИТЕКТУРЫ',
      title: 'Фундамент: 100% безопасный Rust и Apache Arrow',
      lead: 'Статистический пакет не имеет права аварийно падать посреди расчета или замалчивать ошибки NaN. StatCraft следует строгим контрактам надежности памяти и разделения слоев.',
      layerHeading: 'Слоистая архитектура (L0 → L6)',
      layerDesc: 'Проверяется утилитой cargo xtask layers при компиляции. Нулевая циклическая связанность, полная изоляция математики от UI.',
      pillar1Title: 'Контракт нулевых паник',
      pillar1Desc: 'В коде строго запрещены unwrap(), expect(), panic!() и unsafe. Любые нештатные ситуации (вырожденная матрица, деление на ноль, малая выборка) возвращаются типизированными Result<T, StatError>.',
      pillar2Title: 'Принцип чистой комнаты (Clean-Room)',
      pillar2Desc: 'Разработка ведется без заимствования проприетарного кода IBM SPSS. Все алгоритмы воспроизводятся исключительно по открытым научным статьям и спецификациям GNU PSPP.',
      pillar3Title: 'Колоночная память Apache Arrow',
      pillar3Desc: 'Массивы данных хранятся в непрерывных блоках памяти Apache Arrow. Векторные преобразования и фильтрация выполняются движком Polars с поддержкой SIMD без пауз сборщика мусора.',
      pillar4Title: 'Векторная дизайн-система',
      pillar4Desc: 'Нативный интерфейс на egui/eframe. Компонентный DialogKit со статическим линтером Design Guard, субмиллисекундным откликом и защитой от битых глифов Unicode.',
    },
  }[lang];

  const layers = [
    { code: 'L6', name: 'statcraft_app', desc: 'Desktop GUI (egui / eframe / egui_dock / DialogKit)', color: 'text-rose-400' },
    { code: 'L5', name: 'statcraft_syntax', desc: 'CommandBus, .sps parser & command history', color: 'text-amber' },
    { code: 'L4', name: 'statcraft_reporting', desc: 'Pivot Tables model, export (PDF, DOCX, LaTeX)', color: 'text-yellow-400' },
    { code: 'L3', name: 'statcraft_charts', desc: 'Vector charts (egui_plot, brushing & linking)', color: 'text-emerald' },
    { code: 'L2', name: 'statcraft_algo', desc: 'Numerical tests, linear algebra (statrs, faer)', color: 'text-blue-400' },
    { code: 'L1', name: 'statcraft_core', desc: 'Apache Arrow / Polars DataSet & VariableDictionary', color: 'text-indigo-400' },
    { code: 'L0', name: 'statcraft_io', desc: 'Codecs: .stc Parquet, SPSS .sav, CSV, Excel', color: 'text-purple-400' },
  ];

  return (
    <SectionShell id="architecture" eyebrow={t.eyebrow} className="bg-bg">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="mb-10">
          <h2 className="text-2xl sm:text-4xl font-bold text-white tracking-tight mb-4">
            {t.title}
          </h2>
          <p className="text-text-muted text-base sm:text-lg max-w-3xl leading-relaxed">
            {t.lead}
          </p>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-12 gap-8 items-start">
          {/* 4 Pillars Grid */}
          <div className="lg:col-span-7 grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div className="p-5 rounded border border-line bg-bg-raised/40">
              <div className="w-8 h-8 rounded bg-rose-500/10 border border-rose-500/20 flex items-center justify-center text-rose-400 mb-3">
                <ShieldAlert className="w-4 h-4" />
              </div>
              <h3 className="font-semibold text-white text-base mb-1.5">{t.pillar1Title}</h3>
              <p className="text-xs text-text-muted leading-relaxed">
                {t.pillar1Desc}
              </p>
            </div>

            <div className="p-5 rounded border border-line bg-bg-raised/40">
              <div className="w-8 h-8 rounded bg-emerald/10 border border-emerald/20 flex items-center justify-center text-emerald mb-3">
                <CheckCircle2 className="w-4 h-4" />
              </div>
              <h3 className="font-semibold text-white text-base mb-1.5">{t.pillar2Title}</h3>
              <p className="text-xs text-text-muted leading-relaxed">
                {t.pillar2Desc}
              </p>
            </div>

            <div className="p-5 rounded border border-line bg-bg-raised/40">
              <div className="w-8 h-8 rounded bg-blue-500/10 border border-blue-500/20 flex items-center justify-center text-blue-400 mb-3">
                <Cpu className="w-4 h-4" />
              </div>
              <h3 className="font-semibold text-white text-base mb-1.5">{t.pillar3Title}</h3>
              <p className="text-xs text-text-muted leading-relaxed">
                {t.pillar3Desc}
              </p>
            </div>

            <div className="p-5 rounded border border-line bg-bg-raised/40">
              <div className="w-8 h-8 rounded bg-amber/10 border border-amber/20 flex items-center justify-center text-amber mb-3">
                <Sparkles className="w-4 h-4" />
              </div>
              <h3 className="font-semibold text-white text-base mb-1.5">{t.pillar4Title}</h3>
              <p className="text-xs text-text-muted leading-relaxed">
                {t.pillar4Desc}
              </p>
            </div>
          </div>

          {/* Layer Hierarchy Blueprint */}
          <div className="lg:col-span-5 p-6 rounded border border-line bg-[#0c0d10]">
            <div className="flex items-center gap-2 mb-2">
              <Layers className="w-4 h-4 text-blue-400" />
              <h3 className="font-semibold text-white text-sm">{t.layerHeading}</h3>
            </div>
            <p className="text-xs text-text-muted mb-5 leading-relaxed">
              {t.layerDesc}
            </p>

            <div className="space-y-2 font-mono text-xs">
              {layers.map((l) => (
                <div
                  key={l.code}
                  className="p-2.5 rounded bg-[#14161a] border border-line flex items-center justify-between"
                >
                  <div className="flex items-center gap-2.5">
                    <span className={`font-bold ${l.color}`}>{l.code}</span>
                    <span className="text-white font-medium">{l.name}</span>
                  </div>
                  <span className="text-[11px] text-text-muted truncate max-w-[180px] sm:max-w-[220px]">
                    {l.desc}
                  </span>
                </div>
              ))}
            </div>

            <div className="mt-4 pt-4 border-t border-line/60 flex items-center justify-between text-[11px] font-mono text-text-muted">
              <span>Gate: cargo xtask layers</span>
              <span className="text-emerald">Pass (Zero violations)</span>
            </div>
          </div>
        </div>
      </div>
    </SectionShell>
  );
}
