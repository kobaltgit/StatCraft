"use client";

import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";
import { ExternalLink, AlertTriangle, BookOpen, Menu, X, Cpu, Binary, Layers, Users } from "lucide-react";
import React, { useState } from "react";
import { ModalReportIssue } from "./modal-report-issue";
import { DownloadButton } from "./download-button";
import { useRelease } from "@/lib/use-release";

interface SiteNavProps {
  lang?: "en" | "ru";
}

export function SiteNav({ lang = "en" }: SiteNavProps) {
  const pathname = usePathname() || "/";
  const router = useRouter();
  const { tagName } = useRelease();
  const isRu = lang === "ru" || pathname.startsWith("/ru");
  const [modalOpen, setModalOpen] = useState(false);
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);

  // Target path when switching languages
  const alternateLangPath = isRu
    ? pathname.replace(/^\/ru(\/|$)/, "$1") || "/"
    : `/ru${pathname === "/" ? "" : pathname}`;

  const navLinks = [
    {
      id: "manifest",
      label: isRu ? "Манифест" : "Manifest",
      href: isRu ? "/ru#manifest" : "/#manifest",
      icon: <BookOpen className="w-4 h-4 text-amber" />,
    },
    {
      id: "benchmark",
      label: isRu ? "Оракул 20 блоков" : "Oracle Matrix",
      href: isRu ? "/ru#benchmark" : "/#benchmark",
      icon: <Binary className="w-4 h-4 text-emerald" />,
    },
    {
      id: "ai-lab",
      label: isRu ? "ИИ & MCP" : "AI & MCP",
      href: isRu ? "/ru#ai-lab" : "/#ai-lab",
      icon: <Cpu className="w-4 h-4 text-blue-400" />,
    },
    {
      id: "why-rust",
      label: isRu ? "Safe Rust" : "Safe Rust",
      href: isRu ? "/ru#why-rust" : "/#why-rust",
      icon: <Layers className="w-4 h-4 text-purple-400" />,
    },
    {
      id: "researchers",
      label: isRu ? "Сотрудничество" : "Collaboration",
      href: isRu ? "/ru#researchers" : "/#researchers",
      icon: <Users className="w-4 h-4 text-amber" />,
    },
  ];

  const handleNavClick = (e: React.MouseEvent<HTMLAnchorElement>, id: string, targetHref: string) => {
    setMobileMenuOpen(false);

    // If currently on home page (/ or /ru)
    const isHomePage = pathname === "/" || pathname === "/ru";
    if (isHomePage) {
      e.preventDefault();
      const el = document.getElementById(id);
      if (el) {
        el.scrollIntoView({ behavior: "smooth" });
        window.history.pushState(null, "", `#${id}`);
      } else {
        router.push(targetHref);
      }
    }
  };

  return (
    <>
      <nav className="sticky top-0 left-0 z-40 w-full border-b border-line bg-bg/95 backdrop-blur-md">
        <div className="flex h-12 items-stretch justify-between">
          {/* Left: Brand Identity & Mobile Hamburger */}
          <div className="flex items-stretch">
            {/* Mobile Hamburger Button */}
            <button
              type="button"
              onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
              className="lg:hidden flex items-center justify-center px-3.5 border-r border-line text-text-muted hover:text-white"
              aria-label="Toggle menu"
            >
              {mobileMenuOpen ? <X className="w-5 h-5" /> : <Menu className="w-5 h-5" />}
            </button>

            <Link
              href={isRu ? "/ru" : "/"}
              className="flex items-center gap-3 border-r border-line px-4 sm:px-5 hover:opacity-85 transition-opacity"
            >
              {/* eslint-disable-next-line @next/next/no-img-element */}
              <img
                src="/logo.svg"
                alt="StatCraft"
                className="h-6 w-auto"
              />
              <span className="font-mono text-xs font-semibold tracking-wider text-white">
                STATCRAFT
              </span>
              <span className="hidden sm:inline-block text-[10px] px-2 py-0.5 rounded-full bg-accent/20 text-blue-300 border border-accent/40 font-mono">
                {tagName}
              </span>
            </Link>

            {/* Desktop Nav Links */}
            <div className="hidden lg:flex items-stretch">
              {navLinks.map((link) => (
                <a
                  key={link.id}
                  href={link.href}
                  onClick={(e) => handleNavClick(e, link.id, link.href)}
                  className="flex items-center px-3.5 xl:px-4 text-xs font-mono text-text-muted hover:text-white hover:bg-bg-raised transition-colors cursor-pointer"
                >
                  {link.label}
                </a>
              ))}

              <Link
                href={isRu ? "/ru/docs" : "/docs"}
                className="flex items-center gap-1.5 px-3.5 xl:px-4 text-xs font-mono text-blue-400 hover:text-white hover:bg-accent/20 transition-colors font-semibold"
              >
                <BookOpen className="h-3 w-3" />
                <span>{isRu ? "Вики / Документация" : "Docs / Wiki"}</span>
              </Link>
            </div>
          </div>

          {/* Right: Actions, Language Switcher & GitHub */}
          <div className="flex items-stretch">
            {/* Report Discrepancy Button */}
            <button
              type="button"
              onClick={() => setModalOpen(true)}
              className="flex items-center gap-1.5 px-3 sm:px-3.5 border-l border-line text-xs font-medium text-amber hover:bg-amber/10 transition-colors cursor-pointer"
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
                onClick={() => {
                  document.cookie = "statcraft_lang=en; path=/; max-age=31536000; SameSite=Lax";
                }}
                className={`px-1.5 py-0.5 rounded ${!isRu ? "bg-accent text-white font-bold" : "text-text-muted hover:text-white"}`}
              >
                EN
              </Link>
              <span className="text-line mx-1">/</span>
              <Link
                href={isRu ? pathname : alternateLangPath}
                onClick={() => {
                  document.cookie = "statcraft_lang=ru; path=/; max-age=31536000; SameSite=Lax";
                }}
                className={`px-1.5 py-0.5 rounded ${isRu ? "bg-accent text-white font-bold" : "text-text-muted hover:text-white"}`}
              >
                RU
              </Link>
            </div>

            {/* GitHub Repo */}
            <a
              href="https://github.com/kobaltgit/StatCraft"
              target="_blank"
              rel="noreferrer"
              className="flex items-center gap-1.5 border-l border-line px-3 sm:px-4 text-xs font-mono text-text-muted hover:text-white hover:bg-bg-raised transition-colors"
            >
              <span className="hidden sm:inline">GitHub</span>
              <ExternalLink className="h-3 w-3" />
            </a>

            {/* Download Latest Release Button with OS Auto-detect & Direct Download */}
            <DownloadButton lang={isRu ? "ru" : "en"} variant="nav" />
          </div>
        </div>

        {/* Mobile Dropdown Menu Drawer */}
        {mobileMenuOpen && (
          <div className="lg:hidden border-t border-line bg-bg-sunken shadow-2xl px-4 py-4 space-y-3 animate-in fade-in slide-in-from-top-2 duration-200">
            <div className="space-y-1">
              <div className="text-[11px] font-mono text-text-muted uppercase px-2 py-1">
                {isRu ? "Разделы" : "Navigation"}
              </div>

              {navLinks.map((link) => (
                <a
                  key={link.id}
                  href={link.href}
                  onClick={(e) => handleNavClick(e, link.id, link.href)}
                  className="flex items-center gap-2.5 px-2.5 py-2 rounded text-sm text-text-muted hover:text-white hover:bg-bg-raised transition-colors cursor-pointer"
                >
                  {link.icon}
                  <span>{link.label}</span>
                </a>
              ))}

              <Link
                href={isRu ? "/ru/docs" : "/docs"}
                onClick={() => setMobileMenuOpen(false)}
                className="flex items-center gap-2.5 px-2.5 py-2 rounded text-sm text-blue-400 font-semibold hover:bg-accent/20 transition-colors"
              >
                <BookOpen className="w-4 h-4 text-blue-400" />
                <span>{isRu ? "База знаний (Вики)" : "Documentation / Wiki"}</span>
              </Link>
            </div>

            <div className="pt-3 border-t border-line flex flex-col gap-2">
              <button
                type="button"
                onClick={() => {
                  setMobileMenuOpen(false);
                  setModalOpen(true);
                }}
                className="w-full flex items-center justify-center gap-2 px-3 py-2 rounded bg-amber/10 border border-amber/20 text-amber text-xs font-mono cursor-pointer hover:bg-amber/20 transition-colors"
              >
                <AlertTriangle className="w-3.5 h-3.5" />
                <span>{isRu ? "Сообщить о расхождении в расчетах" : "Report Numerical Discrepancy"}</span>
              </button>

              <DownloadButton lang={isRu ? "ru" : "en"} variant="mobile" />
            </div>
          </div>
        )}
      </nav>

      {/* Interactive Issue Reporting Modal */}
      <ModalReportIssue
        isOpen={modalOpen}
        onClose={() => setModalOpen(false)}
        lang={isRu ? "ru" : "en"}
      />
    </>
  );
}
