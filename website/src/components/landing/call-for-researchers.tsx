'use client';

import React, { useState } from 'react';
import { SectionShell } from '../section-shell';
import { ModalReportIssue } from '../modal-report-issue';
import { FlaskConical, Code2, ShieldAlert, HeartHandshake, ArrowRight, MessageSquarePlus } from 'lucide-react';

interface CallForResearchersProps {
  lang?: 'en' | 'ru';
}

export function CallForResearchers({ lang = 'en' }: CallForResearchersProps) {
  const [reportModalOpen, setReportModalOpen] = useState(false);

  const t = {
    en: {
      eyebrow: '05 / INVITATION TO COLLABORATE',
      title: 'An Open Invitation to Researchers & Developers',
      lead: 'We do not claim StatCraft is a ready-made commercial replacement. It is an open-science computational experiment. We need your domain expertise, your datasets, and your scrutiny to make scientific computing transparent and reproducible.',
      resCardTitle: 'For Statisticians & Researchers',
      resCardLead: 'Challenge our numbers. Run your empirical datasets through StatCraft and compare results against SPSS, R, or Stata.',
      resPoint1: 'Test edge cases: ill-conditioned matrices, collinear predictors, tiny frequencies, and weighted survey strata.',
      resPoint2: 'Privacy First: never upload raw patient or proprietary data. Submit anonymized mock cases or summary correlation matrices.',
      resPoint3: 'Found a delta > 10⁻⁶? File an issue directly from the site without needing a GitHub account.',
      resCta: 'Report a Numerical Discrepancy',
      devCardTitle: 'For Rust & Systems Engineers',
      devCardLead: 'Help us architect high-performance, safe computational foundations using modern systems programming.',
      devPoint1: 'Optimize linear algebra routines using faer and statrs without dropping to unsafe.',
      devPoint2: 'Contribute new statistical procedures with SciPy oracle regression tests.',
      devPoint3: 'Expand the Model Context Protocol (MCP) toolset for autonomous AI research pipelines.',
      devCta: 'Explore GitHub Issues & RFCs',
      quote: '«Open science cannot exist on proprietary black-box software. Reproducibility demands open code, open algorithms, and relentless peer verification.»',
    },
    ru: {
      eyebrow: '05 / ПРИГЛАШЕНИЕ К СОТРУДНИЧЕСТВУ',
      title: 'Открытое приглашение исследователям и разработчикам',
      lead: 'Мы не заявляем, что StatCraft — это готовая замена коммерческим пакетам. Это открытый вычислительный эксперимент. Нам нужен ваш исследовательский опыт, ваши датасеты и ваша придирчивость, чтобы сделать научные вычисления прозрачными и воспроизводимыми.',
      resCardTitle: 'Для исследователей и статистиков',
      resCardLead: 'Бросьте вызов нашим расчетам. Прогоните ваши рабочие массивы через StatCraft и сравните сводные таблицы с SPSS, R или Stata.',
      resPoint1: 'Тестируйте граничные случаи: плохо обусловленные матрицы, мультиколлинеарность, малые частоты и весовые схемы.',
      resPoint2: 'Конфиденциальность превыше всего: никогда не передавайте закрытые персональные данные. Используйте синтетические данные или матрицы ковариаций.',
      resPoint3: 'Нашли расхождение > 10⁻⁶? Отправьте тикет прямо с сайта без регистрации на GitHub.',
      resCta: 'Сообщить о расхождении в расчетах',
      devCardTitle: 'Для Rust-инженеров и разработчиков',
      devCardLead: 'Помогите строить надежную вычислительную платформу на передовых инструментах безопасного системного программирования.',
      devPoint1: 'Оптимизируйте матричную алгебру на faer и statrs с нулевым использованием unsafe.',
      devPoint2: 'Внедряйте новые математические процедуры с эталонными тестами оракула SciPy.',
      devPoint3: 'Расширяйте протокол Model Context Protocol (MCP) для автономных цепочек анализа данных.',
      devCta: 'Смотреть задачи на GitHub',
      quote: '«Открытая наука не может опираться на закрытые "черные ящики". Воспроизводимость требует открытого кода, открытых формул и честной перепроверки сообществом.»',
    },
  }[lang];

  return (
    <SectionShell id="researchers" eyebrow={t.eyebrow} className="bg-bg-sunken">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="mb-12">
          <h2 className="text-2xl sm:text-4xl font-bold text-white tracking-tight mb-4">
            {t.title}
          </h2>
          <p className="text-text-muted text-base sm:text-lg max-w-3xl leading-relaxed">
            {t.lead}
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-8 mb-12">
          {/* Researchers Column */}
          <div className="p-6 rounded border border-line bg-bg-raised/40 flex flex-col justify-between">
            <div>
              <div className="w-10 h-10 rounded bg-amber/10 border border-amber/20 flex items-center justify-center text-amber mb-4">
                <FlaskConical className="w-5 h-5" />
              </div>
              <h3 className="text-xl font-bold text-white mb-2">{t.resCardTitle}</h3>
              <p className="text-sm text-text-muted mb-6 leading-relaxed">
                {t.resCardLead}
              </p>

              <ul className="space-y-3 text-xs sm:text-sm text-text-muted mb-8">
                <li className="flex items-start gap-2.5">
                  <span className="w-1.5 h-1.5 rounded-full bg-amber shrink-0 mt-2" />
                  <span>{t.resPoint1}</span>
                </li>
                <li className="flex items-start gap-2.5">
                  <span className="w-1.5 h-1.5 rounded-full bg-amber shrink-0 mt-2" />
                  <span>{t.resPoint2}</span>
                </li>
                <li className="flex items-start gap-2.5">
                  <span className="w-1.5 h-1.5 rounded-full bg-amber shrink-0 mt-2" />
                  <span>{t.resPoint3}</span>
                </li>
              </ul>
            </div>

            <button
              type="button"
              onClick={() => setReportModalOpen(true)}
              className="w-full inline-flex items-center justify-center gap-2 px-4 py-3 rounded bg-amber/10 text-amber hover:bg-amber/20 border border-amber/30 text-sm font-medium transition-colors"
            >
              <MessageSquarePlus className="w-4 h-4" />
              {t.resCta}
            </button>
          </div>

          {/* Developers Column */}
          <div className="p-6 rounded border border-line bg-bg-raised/40 flex flex-col justify-between">
            <div>
              <div className="w-10 h-10 rounded bg-blue-500/10 border border-blue-500/20 flex items-center justify-center text-blue-400 mb-4">
                <Code2 className="w-5 h-5" />
              </div>
              <h3 className="text-xl font-bold text-white mb-2">{t.devCardTitle}</h3>
              <p className="text-sm text-text-muted mb-6 leading-relaxed">
                {t.devCardLead}
              </p>

              <ul className="space-y-3 text-xs sm:text-sm text-text-muted mb-8">
                <li className="flex items-start gap-2.5">
                  <span className="w-1.5 h-1.5 rounded-full bg-blue-400 shrink-0 mt-2" />
                  <span>{t.devPoint1}</span>
                </li>
                <li className="flex items-start gap-2.5">
                  <span className="w-1.5 h-1.5 rounded-full bg-blue-400 shrink-0 mt-2" />
                  <span>{t.devPoint2}</span>
                </li>
                <li className="flex items-start gap-2.5">
                  <span className="w-1.5 h-1.5 rounded-full bg-blue-400 shrink-0 mt-2" />
                  <span>{t.devPoint3}</span>
                </li>
              </ul>
            </div>

            <a
              href="https://github.com/kobaltgit/StatCraft"
              target="_blank"
              rel="noopener noreferrer"
              className="w-full inline-flex items-center justify-center gap-2 px-4 py-3 rounded bg-accent text-white hover:bg-accent-hover text-sm font-medium transition-colors"
            >
              {t.devCta}
              <ArrowRight className="w-4 h-4" />
            </a>
          </div>
        </div>

        {/* Academic Manifesto Quote */}
        <div className="p-6 rounded border border-line/60 bg-[#0c0d10] text-center max-w-3xl mx-auto">
          <HeartHandshake className="w-6 h-6 text-emerald mx-auto mb-3" />
          <blockquote className="text-sm sm:text-base italic text-white/90 leading-relaxed mb-3">
            {t.quote}
          </blockquote>
          <div className="text-xs font-mono text-text-muted">
            StatCraft Open-Science Project // MIT OR Apache-2.0
          </div>
        </div>
      </div>

      <ModalReportIssue
        isOpen={reportModalOpen}
        onClose={() => setReportModalOpen(false)}
        lang={lang}
      />
    </SectionShell>
  );
}
