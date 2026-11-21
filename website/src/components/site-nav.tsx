"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { Download, ExternalLink, AlertTriangle, BookOpen } from "lucide-react";
import React, { useState } from "react";
import { ModalReportIssue } from "./modal-report-issue";

interface SiteNavProps {
  lang?: "en" | "ru";
}

export function SiteNav({ lang = "en" }: SiteNavProps) {
  const pathname = usePathname();
  const isRu = lang === "ru" || pathname.startsWith("/ru");
  const [modalOpen, setModalOpen] = useState(false);

  // Target path when switching languages
  const alternateLangPath = isRu
    ? pathname.replace(/^\/ru/, "") || "/"
    : `/ru${pathname === "/" ? "" : pathname}`;

  return (
    <>
      <nav className="sticky top-0 left-0 z-40 w-full border-b border-line bg-bg/95 backdrop-blur-md">
        <div className="flex h-12 items-stretch justify-between">
          {/* Left: Brand Identity */}
          <div className="flex items-stretch">
            <Link
              href={isRu ? "/ru" : "/"}
              className="flex items-center gap-3 border-r border-line px-5 hover:opacity-85 transition-opacity"
            >
              {/* eslint-disable-next-line @next/next/no-img-element */}
              <img
                src="/logo.svg"
                alt="StatCraft"
                className="h-6 w-auto"
              />
              <span className="font-mono text-xs font-semibold tracking-wider text-ink-strong">
                STATCRAFT
              </span>
              <span className="text-[10px] px-2 py-0.5 rounded-full bg-accent/20 text-accent-ink border border-accent/40 font-mono">
                v0.4.3
              </span>
            </Link>

            {/* Desktop Nav Links */}
            <div className="hidden lg:flex items-stretch">
              <Link
                href={isRu ? "/ru#manifest" : "#manifest"}
                className="flex items-center px-4 hud-label text-muted hover:text-ink hover:bg-invert-bg hover:text-invert-fg transition-colors"
              >
                {isRu ? "Манифест" : "Manifest"}
              </Link>
              <Link
                href={isRu ? "/ru#benchmark" : "#benchmark"}
                className="flex items-center px-4 hud-label text-muted hover:text-ink hover:bg-invert-bg hover:text-invert-fg transition-colors"
              >
                {isRu ? "Оракул 20 блоков" : "Oracle Matrix"}
              </Link>
              <Link
                href={isRu ? "/ru#ai-lab" : "#ai-lab"}
                className="flex items-center px-4 hud-label text-muted hover:text-ink hover:bg-invert-bg hover:text-invert-fg transition-colors"
              >
                {isRu ? "ИИ & MCP" : "AI & MCP"}
              </Link>
              <Link
                href={isRu ? "/ru/docs" : "/docs"}
                className="flex items-center gap-1.5 px-4 hud-label text-accent-ink hover:text-white hover:bg-accent transition-colors font-semibold"
              >
                <BookOpen className="h-3 w-3" />
                <span>{isRu ? "База знаний (Вики)" : "Documentation"}</span>
              </Link>
            </div>
          </div>

          {/* Right: Actions, Language Switcher & GitHub */}
          <div className="flex items-stretch">
            {/* Report Discrepancy Button */}
            <button
              onClick={() => setModalOpen(true)}
              className="flex items-center gap-1.5 px-3.5 border-l border-line text-xs font-medium text-ordinal hover:bg-ordinal/10 transition-colors"
            >
              <AlertTriangle className="h-3.5 w-3.5" />
              <span className="hidden sm:inline">
                {isRu ? "Сообщить о расхождении" : "Report Discrepancy"}
              </span>
            </button>

            {/* Language Switcher */}
            <div className="flex items-center px-3 border-l border-line text-xs font-mono">
              <Link
                href={isRu ? alternateLangPath : pathname}
                className={`px-1.5 py-0.5 rounded ${!isRu ? "bg-accent text-white font-bold" : "text-muted hover:text-ink"}`}
              >
                EN
              </Link>
              <span className="text-faint mx-1">/</span>
              <Link
                href={isRu ? pathname : alternateLangPath}
                className={`px-1.5 py-0.5 rounded ${isRu ? "bg-accent text-white font-bold" : "text-muted hover:text-ink"}`}
              >
                RU
              </Link>
            </div>

            {/* GitHub Repo */}
            <a
              href="https://github.com/kobaltgit/StatCraft"
              target="_blank"
              rel="noreferrer"
              className="flex items-center gap-1.5 border-l border-line px-4 text-xs font-mono text-muted hover:text-ink hover:bg-bg-raised transition-colors"
            >
              <span>GitHub</span>
              <ExternalLink className="h-3 w-3" />
            </a>

            {/* Download Latest Release Button */}
            <a
              href="https://github.com/kobaltgit/StatCraft/releases"
              target="_blank"
              rel="noreferrer"
              className="hidden md:flex items-center gap-2 border-l border-line bg-accent px-5 font-mono text-xs font-semibold text-white hover:bg-accent/90 transition-colors"
            >
              <Download className="h-3.5 w-3.5" />
              <span>{isRu ? "Сборка v0.4.3" : "Release v0.4.3"}</span>
            </a>
          </div>
        </div>
      </nav>

      {/* Interactive Modal */}
      <ModalReportIssue
        isOpen={modalOpen}
        onClose={() => setModalOpen(false)}
        lang={isRu ? "ru" : "en"}
      />
    </>
  );
}
