'use client';

import React from "react";
import Link from "next/link";
import { ExternalLink, ShieldCheck, Download, Coffee } from "lucide-react";
import { useRelease } from "@/lib/use-release";

interface SiteFooterProps {
  lang?: "en" | "ru";
}

export function SiteFooter({ lang = "en" }: SiteFooterProps) {
  const isRu = lang === "ru";
  const { tagName, primaryAsset, osLabelRu, osLabelEn } = useRelease();
  const osLabel = isRu ? osLabelRu : osLabelEn;

  return (
    <footer className="relative border-t border-line bg-bg-sunken text-muted text-xs">
      {/* Top Hairline crosshairs */}
      <div className="relative mx-auto max-w-[1280px] border-x border-line px-6 py-12 md:px-10">
        <span aria-hidden className="tick tick-half -top-[5px] -left-[6px]" />
        <span aria-hidden className="tick tick-half -top-[5px] -right-[5px]" />

        <div className="grid grid-cols-1 md:grid-cols-4 gap-8">
          {/* Col 1: Brand & Charter */}
          <div className="md:col-span-2 space-y-3">
            <div className="flex items-center gap-2">
              <span className="font-mono text-sm font-bold tracking-wider text-ink-strong">
                STATCRAFT
              </span>
              <span className="text-[10px] px-2 py-0.5 rounded bg-accent/20 text-accent-ink border border-accent/30 font-mono">
                100% Safe Rust
              </span>
            </div>
            <p className="text-xs text-muted max-w-md leading-relaxed">
              {isRu
                ? "Открытый исследовательский эксперимент по созданию высокопроизводительного статистического пакета на чистом Safe Rust без паник и сбоев. Проект экосистемы ArtCraft."
                : "An open research exploration in building a memory-safe, panic-free statistical computing platform in pure Safe Rust. Part of the ArtCraft ecosystem."}
            </p>
            <div className="flex items-center gap-2 text-[11px] text-faint">
              <ShieldCheck className="h-3.5 w-3.5 text-nominal" />
              <span>
                {isRu
                  ? "Лицензия: Двойная MIT OR Apache-2.0 • Чистая комната (Clean-Room)"
                  : "Dual Licensed: MIT OR Apache-2.0 • Clean-Room Methodology"}
              </span>
            </div>
          </div>

          {/* Col 2: Navigation */}
          <div className="space-y-2">
            <div className="hud-label text-ink-strong mb-2">
              {isRu ? "Навигация" : "Navigation"}
            </div>
            <ul className="space-y-1.5 text-xs">
              <li>
                <Link href={isRu ? "/ru" : "/"} className="hover:text-ink transition">
                  {isRu ? "Главная / Манифест" : "Home / Manifest"}
                </Link>
              </li>
              <li>
                <Link href={isRu ? "/ru#benchmark" : "/#benchmark"} className="hover:text-ink transition">
                  {isRu ? "Оракул 20 блоков" : "Oracle Matrix"}
                </Link>
              </li>
              <li>
                <Link href={isRu ? "/ru/docs" : "/docs"} className="hover:text-ink transition">
                  {isRu ? "База знаний (Вики)" : "Documentation (Wiki)"}
                </Link>
              </li>
              <li>
                <a
                  href={primaryAsset.downloadUrl}
                  download
                  className="hover:text-ink transition inline-flex items-center gap-1.5 text-accent font-mono"
                  title={`${isRu ? 'Скачать бинарник' : 'Download binary'}: ${primaryAsset.name}`}
                >
                  <Download className="h-3 w-3" />
                  <span>{isRu ? `Скачать ${tagName} (${osLabel})` : `Download ${tagName} (${osLabel})`}</span>
                </a>
              </li>
              <li>
                <a
                  href="https://github.com/kobaltgit/StatCraft/releases"
                  target="_blank"
                  rel="noreferrer"
                  className="hover:text-ink transition inline-flex items-center gap-1"
                >
                  <span>{isRu ? "Все релизы на GitHub" : "All GitHub Releases"}</span>
                  <ExternalLink className="h-3 w-3" />
                </a>
              </li>
            </ul>
          </div>

          {/* Col 3: Community & Issues */}
          <div className="space-y-2">
            <div className="hud-label text-ink-strong mb-2">
              {isRu ? "Сообщество" : "Community"}
            </div>
            <ul className="space-y-1.5 text-xs">
              <li>
                <a
                  href="https://github.com/kobaltgit/StatCraft"
                  target="_blank"
                  rel="noreferrer"
                  className="hover:text-ink transition inline-flex items-center gap-1"
                >
                  <span>GitHub Repository</span>
                  <ExternalLink className="h-3 w-3" />
                </a>
              </li>
              <li>
                <a
                  href="https://github.com/kobaltgit/StatCraft/issues"
                  target="_blank"
                  rel="noreferrer"
                  className="hover:text-ink transition inline-flex items-center gap-1"
                >
                  <span>GitHub Issues</span>
                  <ExternalLink className="h-3 w-3" />
                </a>
              </li>
              <li>
                <a
                  href="https://app.lava.top/kobaltgit"
                  target="_blank"
                  rel="noreferrer"
                  className="hover:text-amber text-amber/90 transition inline-flex items-center gap-1.5"
                >
                  <Coffee className="h-3 w-3 text-amber" />
                  <span>{isRu ? "Поддержать (Lava.top)" : "Buy me a Coffee"}</span>
                  <ExternalLink className="h-2.5 w-2.5 opacity-60" />
                </a>
              </li>
              <li>
                <a
                  href="https://news.ycombinator.com"
                  target="_blank"
                  rel="noreferrer"
                  className="hover:text-ink transition inline-flex items-center gap-1"
                >
                  <span>Hacker News Discussion</span>
                  <ExternalLink className="h-3 w-3" />
                </a>
              </li>
            </ul>
          </div>
        </div>

        {/* Bottom Hairline & Disclaimer */}
        <div className="mt-8 pt-6 border-t border-line flex flex-col md:flex-row items-center justify-between gap-4 text-[11px] text-faint">
          <div>
            © {new Date().getFullYear()} kobaltgit & StatCraft contributors. Designed under ArtCraft standards.
          </div>
          <div className="text-right">
            {isRu
              ? "Экспериментальная версия: не для критических мед./юр. решений без верификации."
              : "Experimental project: verify critical analytical calculations with reference packages."}
          </div>
        </div>
      </div>
    </footer>
  );
}
