'use client';

import React, { useState } from 'react';
import { SiteNav } from '../site-nav';
import { SiteFooter } from '../site-footer';
import { DocsSidebar } from './docs-sidebar';
import { Menu, X } from 'lucide-react';
import { useRelease } from '@/lib/use-release';

interface DocsShellProps {
  children: React.ReactNode;
  lang?: 'en' | 'ru';
}

export function DocsShell({ children, lang = 'en' }: DocsShellProps) {
  const [mobileSidebarOpen, setMobileSidebarOpen] = useState(false);
  const { tagName } = useRelease();

  return (
    <div className="min-h-screen flex flex-col bg-bg text-text">
      <SiteNav lang={lang} />

      {/* Mobile sidebar toggle bar */}
      <div className="lg:hidden px-4 py-2.5 bg-bg-sunken border-b border-line flex items-center justify-between">
        <button
          type="button"
          onClick={() => setMobileSidebarOpen(!mobileSidebarOpen)}
          className="inline-flex items-center gap-2 text-xs font-mono text-text-muted hover:text-white"
        >
          {mobileSidebarOpen ? <X className="w-4 h-4" /> : <Menu className="w-4 h-4" />}
          <span>{lang === 'ru' ? 'Меню разделов вики' : 'Wiki Navigation Menu'}</span>
        </button>
        <span className="text-[11px] font-mono text-text-muted">{tagName}</span>
      </div>

      <div className="flex-1 flex max-w-7xl w-full mx-auto relative">
        {/* Desktop Sidebar (Left Column) */}
        <div className="hidden lg:block w-72 shrink-0 border-r border-line sticky top-14 h-[calc(100vh-3.5rem)]">
          <DocsSidebar lang={lang} />
        </div>

        {/* Mobile Sidebar Drawer */}
        {mobileSidebarOpen && (
          <div className="fixed inset-0 z-50 lg:hidden flex">
            <div
              className="fixed inset-0 bg-black/60 backdrop-blur-sm"
              onClick={() => setMobileSidebarOpen(false)}
            />
            <div className="relative w-80 max-w-[85vw] h-full bg-bg-sunken z-10 shadow-2xl">
              <DocsSidebar
                lang={lang}
                onLinkClick={() => setMobileSidebarOpen(false)}
              />
            </div>
          </div>
        )}

        {/* Middle and Right Columns via children */}
        <div className="flex-1 min-w-0">
          {children}
        </div>
      </div>

      <SiteFooter lang={lang} />
    </div>
  );
}
