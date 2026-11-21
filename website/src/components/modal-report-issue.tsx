"use client";

import React, { useState } from "react";
import { AlertCircle, CheckCircle, Send, X, Shield, FileText } from "lucide-react";

interface ModalReportIssueProps {
  isOpen: boolean;
  onClose: () => void;
  lang?: "en" | "ru";
  prefillProcedure?: string;
}

export function ModalReportIssue({
  isOpen,
  onClose,
  lang = "en",
  prefillProcedure,
}: ModalReportIssueProps) {
  const [category, setCategory] = useState("Math / Calculation Discrepancy");
  const [title, setTitle] = useState(prefillProcedure ? `[Audit Discrepancy] ${prefillProcedure}` : "");
  const [description, setDescription] = useState("");
  const [snippet, setSnippet] = useState("");
  const [contact, setContact] = useState("");
  const [honeypot, setHoneypot] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [createdIssue, setCreatedIssue] = useState<{ url: string; number: number } | null>(null);

  if (!isOpen) return null;

  const isRu = lang === "ru";

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setLoading(true);
    setError(null);

    try {
      const res = await fetch("/api/create-issue", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          category,
          title,
          description,
          snippet,
          contact,
          honeypot,
        }),
      });

      const data = await res.json();
      if (!res.ok) {
        throw new Error(data.error || "Failed to submit report");
      }

      setCreatedIssue({ url: data.issue_url, number: data.issue_number });
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Submission failed");
    } finally {
      setLoading(false);
    }
  }

  function handleReset() {
    setTitle("");
    setDescription("");
    setSnippet("");
    setContact("");
    setError(null);
    setCreatedIssue(null);
    onClose();
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm animate-in fade-in duration-200">
      <div className="relative w-full max-w-xl rounded-xl border border-line-strong bg-bg-raised p-6 shadow-2xl text-ink">
        {/* Header with drafting tick */}
        <span aria-hidden className="tick -top-[5px] -left-[5px]" />
        <span aria-hidden className="tick -top-[5px] -right-[5px]" />

        <div className="flex items-center justify-between border-b border-line pb-3.5">
          <div className="flex items-center gap-2.5">
            <div className="flex h-7 w-7 items-center justify-center rounded bg-accent/20 text-accent-ink border border-accent/40">
              <FileText className="h-4 w-4" />
            </div>
            <div>
              <h3 className="text-sm font-semibold tracking-wide text-ink-strong">
                {isRu ? "Сообщить о расхождении или дефекте" : "Report Discrepancy or Defect"}
              </h3>
              <p className="hud-label text-faint text-[10px]">
                {isRu ? "Создание тикета в GitHub без авторизации" : "Direct GitHub Issue submission without login"}
              </p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="rounded p-1 text-muted hover:text-ink hover:bg-bg transition-colors"
          >
            <X className="h-4 w-4" />
          </button>
        </div>

        {createdIssue ? (
          <div className="py-6 text-center space-y-4">
            <div className="inline-flex h-12 w-12 items-center justify-center rounded-full bg-nominal/20 text-nominal border border-nominal/40">
              <CheckCircle className="h-6 w-6" />
            </div>
            <div className="space-y-1">
              <h4 className="text-base font-semibold text-ink-strong">
                {isRu ? "Обращение успешно зарегистрировано!" : "Report Registered Successfully!"}
              </h4>
              <p className="text-xs text-muted">
                {isRu
                  ? `Создан открытый тикет в репозитории: Issue #${createdIssue.number}`
                  : `Created public GitHub issue: Issue #${createdIssue.number}`}
              </p>
            </div>
            <div className="pt-2">
              <a
                href={createdIssue.url}
                target="_blank"
                rel="noreferrer"
                className="inline-flex items-center gap-1.5 px-4 py-2 rounded text-xs font-semibold bg-accent text-white hover:bg-accent/90 transition shadow"
              >
                {isRu ? "Открыть тикет на GitHub" : "View Issue on GitHub"}
              </a>
            </div>
            <div className="pt-2">
              <button
                onClick={handleReset}
                className="text-xs text-muted hover:text-ink underline"
              >
                {isRu ? "Закрыть окно" : "Close Window"}
              </button>
            </div>
          </div>
        ) : (
          <form onSubmit={handleSubmit} className="mt-4 space-y-3.5 text-xs">
            {/* Honeypot field for bot filtering */}
            <input
              type="text"
              name="website_validation_token"
              value={honeypot}
              onChange={(e) => setHoneypot(e.target.value)}
              className="hidden"
              tabIndex={-1}
              autoComplete="off"
            />

            {error && (
              <div className="p-3 rounded bg-red-950/60 border border-red-800 text-red-200 flex items-center gap-2">
                <AlertCircle className="h-4 w-4 shrink-0 text-red-400" />
                <span>{error}</span>
              </div>
            )}

            <div>
              <label className="hud-label text-muted block mb-1">
                {isRu ? "Категория обращения" : "Category"}
              </label>
              <select
                value={category}
                onChange={(e) => setCategory(e.target.value)}
                className="w-full rounded border border-line bg-bg px-3 py-1.5 text-xs text-ink focus:border-accent focus:outline-none"
              >
                <option value="Math / Calculation Discrepancy">
                  {isRu ? "Расхождение в расчетах p-value или формулах" : "Math / Calculation Discrepancy"}
                </option>
                <option value="Data Codec / Import Issue">
                  {isRu ? "Проблема чтения файла (.sav / Excel / CSV)" : "Data Codec / Import Issue"}
                </option>
                <option value="UI & Pivot Tables Defect">
                  {isRu ? "Дефект интерфейса или таблицы вывода" : "UI & Pivot Tables Defect"}
                </option>
                <option value="Feature Suggestion & Question">
                  {isRu ? "Предложение по улучшению или вопрос" : "Feature Suggestion & Question"}
                </option>
              </select>
            </div>

            <div>
              <label className="hud-label text-muted block mb-1">
                {isRu ? "Тема (кратко)" : "Summary Title"}
              </label>
              <input
                type="text"
                required
                placeholder={isRu ? "Например: Различие в df Саттертуэйта на выборках N=12 и N=15" : "e.g., Satterthwaite df difference on N=12 and N=15"}
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                className="w-full rounded border border-line bg-bg px-3 py-1.5 text-xs text-ink placeholder-faint focus:border-accent focus:outline-none"
              />
            </div>

            <div>
              <label className="hud-label text-muted block mb-1">
                {isRu ? "Подробное описание расхождения" : "Detailed Description"}
              </label>
              <textarea
                required
                rows={3}
                placeholder={isRu ? "Опишите, какие значения получены в StatCraft и какие выдал референтный пакет (SPSS, R, SciPy)..." : "Explain what StatCraft output vs. what SPSS/R/SciPy returned on the same data..."}
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                className="w-full rounded border border-line bg-bg px-3 py-1.5 text-xs text-ink placeholder-faint focus:border-accent focus:outline-none"
              />
            </div>

            <div>
              <label className="hud-label text-muted block mb-1">
                {isRu ? "Фрагмент данных или синтаксиса (опционально)" : "Data or Syntax Snippet (Optional)"}
              </label>
              <textarea
                rows={2}
                placeholder={isRu ? "3-5 строк анонимизированных данных или команда синтаксиса..." : "3-5 rows of sample numbers or syntax command..."}
                value={snippet}
                onChange={(e) => setSnippet(e.target.value)}
                className="w-full font-mono rounded border border-line bg-bg px-3 py-1.5 text-[11px] text-accent-ink placeholder-faint focus:border-accent focus:outline-none"
              />
            </div>

            <div>
              <label className="hud-label text-muted block mb-1">
                {isRu ? "Контакт для обратной связи (Email / соцсеть, опционально)" : "Contact (Email / Handle, Optional)"}
              </label>
              <input
                type="text"
                placeholder="dr_smith@university.edu"
                value={contact}
                onChange={(e) => setContact(e.target.value)}
                className="w-full rounded border border-line bg-bg px-3 py-1.5 text-xs text-ink placeholder-faint focus:border-accent focus:outline-none"
              />
            </div>

            <div className="flex items-center justify-between pt-3 border-t border-line">
              <div className="flex items-center gap-1.5 text-[10px] text-faint">
                <Shield className="h-3 w-3 text-accent" />
                <span>{isRu ? "Защищено Honeypot & IP rate-limit" : "Protected by Honeypot & IP rate-limit"}</span>
              </div>
              <div className="flex gap-2">
                <button
                  type="button"
                  onClick={onClose}
                  className="px-3 py-1.5 rounded border border-line text-muted hover:text-ink hover:bg-bg transition"
                >
                  {isRu ? "Отмена" : "Cancel"}
                </button>
                <button
                  type="submit"
                  disabled={loading}
                  className="inline-flex items-center gap-1.5 px-4 py-1.5 rounded font-semibold bg-accent text-white hover:bg-accent/90 disabled:opacity-50 transition shadow"
                >
                  <Send className="h-3 w-3" />
                  <span>{loading ? (isRu ? "Отправка..." : "Submitting...") : (isRu ? "Отправить тикет" : "Submit Issue")}</span>
                </button>
              </div>
            </div>
          </form>
        )}
      </div>
    </div>
  );
}
