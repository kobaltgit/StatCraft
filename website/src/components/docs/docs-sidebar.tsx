'use client';

import React, { useState, useMemo } from 'react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { DOC_SECTIONS, DocSection } from '@/lib/docs-data';
import { Search, BookOpen, ChevronRight, Binary, ShieldCheck, Bot, Database } from 'lucide-react';

interface DocsSidebarProps {
  lang?: 'en' | 'ru';
  className?: string;
  onLinkClick?: () => void;
}

export function DocsSidebar({ lang = 'en', className = '', onLinkClick }: DocsSidebarProps) {
  const pathname = usePathname();
  const [query, setQuery] = useState('');

  const t = {
    en: {
      searchPlaceholder: 'Search wiki & procedures...',
      categories: {
        'getting-started': 'Getting Started',
        'data-management': 'Data Management & IO',
        'procedures': 'Statistical Procedures',
        'audit': 'Evidence & Math Audit',
        'agent-lab': 'Agent Lab & MCP',
      },
      backToHome: '← Back to Overview',
    },
    ru: {
      searchPlaceholder: 'Поиск по вики и процедурам...',
      categories: {
        'getting-started': 'Начало работы',
        'data-management': 'Управление данными и ввод-вывод',
        'procedures': 'Статистические процедуры',
        'audit': 'Доказательный аудит ядра',
        'agent-lab': 'ИИ-лаборатория и MCP',
      },
      backToHome: '← На главную страницу',
    },
  }[lang];

  const filteredSections = useMemo(() => {
    if (!query.trim()) return DOC_SECTIONS;
    const term = query.toLowerCase();
    return DOC_SECTIONS.filter(s => {
      const title = (lang === 'ru' ? s.titleRu : s.titleEn).toLowerCase();
      const desc = (lang === 'ru' ? s.descriptionRu : s.descriptionEn).toLowerCase();
      return title.includes(term) || desc.includes(term) || s.slug.includes(term);
    });
  }, [query, lang]);

  // Group by category
  const categories = useMemo(() => {
    const map = new Map<string, DocSection[]>();
    filteredSections.forEach(s => {
      const cat = s.category;
      if (!map.has(cat)) map.set(cat, []);
      map.get(cat)!.push(s);
    });
    return map;
  }, [filteredSections]);

  const basePath = lang === 'ru' ? '/ru/docs' : '/docs';

  const categoryIcons: Record<string, React.ReactNode> = {
    'getting-started': <BookOpen className="w-4 h-4 text-amber" />,
    'data-management': <Database className="w-4 h-4 text-indigo-400" />,
    'procedures': <Binary className="w-4 h-4 text-blue-400" />,
    'audit': <ShieldCheck className="w-4 h-4 text-emerald" />,
    'agent-lab': <Bot className="w-4 h-4 text-purple-400" />,
  };

  return (
    <aside className={`w-full h-full flex flex-col border-r border-line bg-bg-sunken ${className}`}>
      {/* Search box */}
      <div className="p-4 border-b border-line">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-text-muted" />
          <input
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder={t.searchPlaceholder}
            className="w-full pl-9 pr-3 py-2 rounded bg-bg border border-line text-xs text-text placeholder:text-text-muted/60 focus:outline-none focus:border-accent"
          />
        </div>
      </div>

      {/* Nav list */}
      <nav className="flex-1 overflow-y-auto p-4 space-y-6">
        <div className="mb-2">
          <Link
            href={lang === 'ru' ? '/ru' : '/'}
            className="text-xs font-mono text-text-muted hover:text-white transition-colors"
          >
            {t.backToHome}
          </Link>
        </div>

        {Array.from(categories.entries()).map(([catKey, items]) => {
          const catLabel = t.categories[catKey as keyof typeof t.categories] || catKey;
          const icon = categoryIcons[catKey] || <BookOpen className="w-4 h-4 text-text-muted" />;

          return (
            <div key={catKey} className="space-y-1.5">
              <div className="flex items-center gap-2 px-2 py-1 text-xs font-mono font-medium text-text-muted uppercase tracking-wider">
                {icon}
                <span>{catLabel}</span>
              </div>

              <div className="space-y-0.5">
                {items.map((item) => {
                  const href = `${basePath}/${item.slug}`;
                  const isActive = pathname === href || (pathname === basePath && item.slug === 'experiment-manifest');
                  const itemTitle = lang === 'ru' ? item.titleRu : item.titleEn;

                  return (
                    <Link
                      key={item.slug}
                      href={href}
                      onClick={onLinkClick}
                      className={`group flex items-center justify-between px-2.5 py-1.5 rounded text-xs transition-colors ${
                        isActive
                          ? 'bg-accent/15 text-white font-medium border-l-2 border-accent'
                          : 'text-text-muted hover:text-white hover:bg-bg-raised/50'
                      }`}
                    >
                      <span className="truncate">{itemTitle}</span>
                      {isActive && (
                        <ChevronRight className="w-3.5 h-3.5 text-accent shrink-0 ml-1" />
                      )}
                    </Link>
                  );
                })}
              </div>
            </div>
          );
        })}
      </nav>

      {/* Sidebar footer status */}
      <div className="p-3 border-t border-line text-[11px] font-mono text-text-muted flex items-center justify-between">
        <span>Build 0.4.3</span>
        <span className="text-emerald">20 / 20 Verified</span>
      </div>
    </aside>
  );
}
