"use client";

import { useEffect, useState } from "react";
import { AUDIT_BLOCKS } from "@/lib/audit-data";

const SECTIONS_EN = [
  { id: "manifest", label: "01 / MANIFEST" },
  { id: "benchmark", label: `02 / ${AUDIT_BLOCKS.length}-BLOCK ORACLE` },
  { id: "ai-lab", label: "03 / AI & MCP LAB" },
  { id: "why-rust", label: "04 / SAFE RUST" },
  { id: "researchers", label: "05 / PARTICIPATE" },
];

const SECTIONS_RU = [
  { id: "manifest", label: "01 / МАНИФЕСТ" },
  { id: "benchmark", label: `02 / ОРАКУЛ ${AUDIT_BLOCKS.length} БЛОКОВ` },
  { id: "ai-lab", label: "03 / ИИ И MCP" },
  { id: "why-rust", label: "04 / SAFE RUST" },
  { id: "researchers", label: "05 / УЧАСТИЕ" },
];

export function FixedCornerNav({ lang = "en" }: { lang?: "en" | "ru" }) {
  const [activeId, setActiveId] = useState("manifest");
  const sections = lang === "ru" ? SECTIONS_RU : SECTIONS_EN;

  useEffect(() => {
    const handleScroll = () => {
      const scrollPos = window.scrollY + 250;
      for (let i = sections.length - 1; i >= 0; i--) {
        const sec = document.getElementById(sections[i].id);
        if (sec && sec.offsetTop <= scrollPos) {
          setActiveId(sections[i].id);
          break;
        }
      }
    };

    window.addEventListener("scroll", handleScroll, { passive: true });
    handleScroll();
    return () => window.removeEventListener("scroll", handleScroll);
  }, [sections]);

  const scrollTo = (id: string) => {
    const el = document.getElementById(id);
    if (el) {
      el.scrollIntoView({ behavior: "smooth" });
    }
  };

  return (
    <aside
      aria-label="Section Navigation"
      className="hidden xl:flex fixed bottom-8 right-8 z-30 flex-col items-end gap-1.5 select-none pointer-events-auto"
    >
      {sections.map((sec) => {
        const isActive = activeId === sec.id;
        return (
          <button
            key={sec.id}
            type="button"
            onClick={() => scrollTo(sec.id)}
            className={`group flex items-center justify-end text-right transition-all duration-200 cursor-pointer ${
              isActive ? "text-ink-strong font-bold" : "text-faint hover:text-ink font-medium"
            }`}
          >
            <span
              className={`font-mono text-[10px] tracking-[0.14em] uppercase transition-colors ${
                isActive ? "text-accent-ink" : "text-muted group-hover:text-ink"
              }`}
            >
              {sec.label}
            </span>
            <span
              aria-hidden
              className={`ml-2.5 h-1 transition-all duration-200 ${
                isActive
                  ? "w-3 bg-accent opacity-100"
                  : "w-1 bg-line-strong opacity-40 group-hover:w-2 group-hover:opacity-80"
              }`}
            />
          </button>
        );
      })}
    </aside>
  );
}
