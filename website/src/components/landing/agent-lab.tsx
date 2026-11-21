'use client';

import React, { useState } from 'react';
import { SectionShell } from '../section-shell';
import { Bot, Terminal, Cpu, Network, CheckCircle2, Copy, Check } from 'lucide-react';

interface AgentLabProps {
  lang?: 'en' | 'ru';
}

export function AgentLab({ lang = 'en' }: AgentLabProps) {
  const [copiedTab, setCopiedTab] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<'mcp' | 'jsonrpc' | 'interpretation'>('mcp');

  const t = {
    en: {
      eyebrow: '03 / AGENT-FIRST PROTOCOL',
      title: 'Model Context Protocol (MCP) & Local IPC Socket 18492',
      lead: 'StatCraft is built from the ground up for the agentic era. Any AI assistant—Antigravity, Claude, or local LLMs—can autonomously inspect variables, execute commands, and write peer-reviewed interpretations directly into the Output Viewer.',
      badgeSocket: 'LOCAL IPC SOCKET: 127.0.0.1:18492',
      badgeStd: 'MODEL CONTEXT PROTOCOL (MCP 1.0)',
      tab1: 'MCP Tool Call',
      tab2: 'Raw JSON-RPC',
      tab3: 'Viewer Interpretation',
      feature1Title: 'Zero-Hallucination Metadata',
      feature1Desc: 'Agents query the exact VariableDictionary directly from RAM. No guessing variable names, types, or missing value codes.',
      feature2Title: 'CommandBus Execution',
      feature2Desc: 'All procedures execute through typed commands with full Undo/Redo tracking and SPSS syntax generation.',
      feature3Title: 'Direct Output Injection',
      feature3Desc: 'AI writes structured APA-style tables and academic interpretations directly into the desktop Output Viewer.',
    },
    ru: {
      eyebrow: '03 / AGENT-FIRST АРХИТЕКТУРА',
      title: 'Model Context Protocol (MCP) и сокет IPC 18492',
      lead: 'StatCraft спроектирован для эпохи ИИ-агентов. Любой ассистент — Antigravity, Claude или локальная LLM — может исследовать переменные в памяти, выполнять команды и вставлять научные интерпретации прямо в Output Viewer.',
      badgeSocket: 'ЛОКАЛЬНЫЙ СОКЕТ IPC: 127.0.0.1:18492',
      badgeStd: 'ПРОТОКОЛ MODEL CONTEXT PROTOCOL (MCP 1.0)',
      tab1: 'Вызов инструмента MCP',
      tab2: 'Сырой JSON-RPC',
      tab3: 'Интерпретация в Output',
      feature1Title: 'Метаданные без галлюцинаций',
      feature1Desc: 'Агенты запрашивают VariableDictionary напрямую из оперативной памяти. Никаких догадок об именах, шкалах и кодах пропусков.',
      feature2Title: 'Выполнение через CommandBus',
      feature2Desc: 'Все процедуры выполняются типизированными командами с поддержкой Undo/Redo и генерацией синтаксиса SPSS.',
      feature3Title: 'Вставка в Output Viewer',
      feature3Desc: 'ИИ вставляет сводные таблицы по стандартам APA и научные выводы прямо в дерево результатов программы.',
    },
  }[lang];

  const snippets = {
    mcp: `{
  "name": "statcraft_execute_syntax",
  "arguments": {
    "syntax": "T-TEST GROUPS=treatment(1, 2) /VARIABLES=cholesterol /CRITERIA=CI(0.95)."
  }
}`,
    jsonrpc: `--> {
  "jsonrpc": "2.0",
  "id": 42,
  "method": "statcraft.procedure.ttest_independent",
  "params": {
    "test_var": "cholesterol",
    "group_var": "treatment",
    "group_val1": 1.0,
    "group_val2": 2.0,
    "confidence": 0.95
  }
}
<-- {
  "jsonrpc": "2.0",
  "id": 42,
  "result": {
    "levene_f": 0.421,
    "levene_p": 0.518,
    "t": 3.142,
    "df": 98.0,
    "p_two_tailed": 0.0022,
    "mean_diff": -14.82,
    "ci_95": [-24.18, -5.46]
  }
}`,
    interpretation: `### Independent Samples t-Test (Treatment vs Control)

An independent-samples t-test was conducted to compare cholesterol levels in control (M = 214.3, SD = 24.1) and treatment (M = 199.5, SD = 22.8) conditions.

* **Levene's Test for Equality of Variances**: F(1, 98) = 0.421, p = .518 (Equal variances assumed).
* **Test Statistic**: t(98) = 3.142, p = .0022, two-tailed.
* **Effect Size**: Cohen's d = 0.63 (medium-to-large effect), 95% CI [-24.18, -5.46].

*Conclusion*: Statistically significant reduction in serum cholesterol in the treatment group.`
  };

  const handleCopy = (text: string, key: string) => {
    navigator.clipboard.writeText(text);
    setCopiedTab(key);
    setTimeout(() => setCopiedTab(null), 2000);
  };

  return (
    <SectionShell id="agent-lab" eyebrow={t.eyebrow} className="bg-bg-sunken">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="mb-10">
          <div className="flex flex-wrap items-center gap-3 mb-4">
            <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded bg-accent/10 border border-accent/20 text-blue-400 text-xs font-mono font-medium">
              <Network className="w-3.5 h-3.5" />
              {t.badgeSocket}
            </span>
            <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded bg-bg-raised border border-line text-text-muted text-xs font-mono">
              <Bot className="w-3.5 h-3.5 text-amber" />
              {t.badgeStd}
            </span>
          </div>

          <h2 className="text-2xl sm:text-4xl font-bold text-white tracking-tight mb-4">
            {t.title}
          </h2>
          <p className="text-text-muted text-base sm:text-lg max-w-3xl leading-relaxed">
            {t.lead}
          </p>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-12 gap-8 items-start">
          {/* Features Column */}
          <div className="lg:col-span-5 space-y-6">
            <div className="p-5 rounded border border-line bg-bg-raised/40">
              <div className="flex items-center gap-3 mb-2">
                <div className="w-8 h-8 rounded bg-blue-500/10 border border-blue-500/20 flex items-center justify-center text-blue-400">
                  <Terminal className="w-4 h-4" />
                </div>
                <h3 className="font-semibold text-white text-base">{t.feature1Title}</h3>
              </div>
              <p className="text-xs sm:text-sm text-text-muted leading-relaxed pl-11">
                {t.feature1Desc}
              </p>
            </div>

            <div className="p-5 rounded border border-line bg-bg-raised/40">
              <div className="flex items-center gap-3 mb-2">
                <div className="w-8 h-8 rounded bg-emerald/10 border border-emerald/20 flex items-center justify-center text-emerald">
                  <Cpu className="w-4 h-4" />
                </div>
                <h3 className="font-semibold text-white text-base">{t.feature2Title}</h3>
              </div>
              <p className="text-xs sm:text-sm text-text-muted leading-relaxed pl-11">
                {t.feature2Desc}
              </p>
            </div>

            <div className="p-5 rounded border border-line bg-bg-raised/40">
              <div className="flex items-center gap-3 mb-2">
                <div className="w-8 h-8 rounded bg-amber/10 border border-amber/20 flex items-center justify-center text-amber">
                  <Bot className="w-4 h-4" />
                </div>
                <h3 className="font-semibold text-white text-base">{t.feature3Title}</h3>
              </div>
              <p className="text-xs sm:text-sm text-text-muted leading-relaxed pl-11">
                {t.feature3Desc}
              </p>
            </div>
          </div>

          {/* Interactive Code Terminal Column */}
          <div className="lg:col-span-7 rounded border border-line bg-[#0c0d10] overflow-hidden shadow-2xl">
            {/* Terminal Window Header */}
            <div className="px-4 py-3 bg-[#14161a] border-b border-line flex items-center justify-between">
              <div className="flex items-center gap-2">
                <div className="w-3 h-3 rounded-full bg-rose-500/80" />
                <div className="w-3 h-3 rounded-full bg-amber/80" />
                <div className="w-3 h-3 rounded-full bg-emerald/80" />
                <span className="ml-2 font-mono text-xs text-text-muted">statcraft-mcp-bridge</span>
              </div>

              {/* Tabs */}
              <div className="flex items-center gap-1">
                <button
                  type="button"
                  onClick={() => setActiveTab('mcp')}
                  className={`px-2.5 py-1 rounded text-xs font-mono transition-colors ${
                    activeTab === 'mcp' ? 'bg-accent text-white' : 'text-text-muted hover:text-white'
                  }`}
                >
                  {t.tab1}
                </button>
                <button
                  type="button"
                  onClick={() => setActiveTab('jsonrpc')}
                  className={`px-2.5 py-1 rounded text-xs font-mono transition-colors ${
                    activeTab === 'jsonrpc' ? 'bg-accent text-white' : 'text-text-muted hover:text-white'
                  }`}
                >
                  {t.tab2}
                </button>
                <button
                  type="button"
                  onClick={() => setActiveTab('interpretation')}
                  className={`px-2.5 py-1 rounded text-xs font-mono transition-colors ${
                    activeTab === 'interpretation' ? 'bg-accent text-white' : 'text-text-muted hover:text-white'
                  }`}
                >
                  {t.tab3}
                </button>
              </div>
            </div>

            {/* Terminal Code Body */}
            <div className="relative p-5">
              <button
                type="button"
                onClick={() => handleCopy(snippets[activeTab], activeTab)}
                className="absolute top-4 right-4 p-2 rounded bg-bg-raised/80 hover:bg-bg-raised text-text-muted hover:text-white border border-line transition-colors"
                title="Copy code"
              >
                {copiedTab === activeTab ? (
                  <Check className="w-4 h-4 text-emerald" />
                ) : (
                  <Copy className="w-4 h-4" />
                )}
              </button>

              <pre className="font-mono text-xs sm:text-sm text-text leading-relaxed overflow-x-auto selection:bg-accent/30">
                <code>{snippets[activeTab]}</code>
              </pre>
            </div>

            {/* Terminal Footer status */}
            <div className="px-4 py-2 bg-[#14161a] border-t border-line text-[11px] font-mono text-text-muted flex items-center justify-between">
              <span className="flex items-center gap-1.5 text-emerald">
                <CheckCircle2 className="w-3.5 h-3.5" />
                <span>Socket 18492 Ready (Async Tokio)</span>
              </span>
              <span>L5 / statcraft_bridge</span>
            </div>
          </div>
        </div>
      </div>
    </SectionShell>
  );
}
