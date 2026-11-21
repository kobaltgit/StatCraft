'use client';

import React, { useState } from 'react';
import { Copy, Check, Terminal } from 'lucide-react';

interface SpssSyntaxBlockProps {
  syntax: string;
  title?: string;
  className?: string;
}

export function SpssSyntaxBlock({ syntax, title = 'SPSS SYNTAX (.SPS)', className = '' }: SpssSyntaxBlockProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = () => {
    navigator.clipboard.writeText(syntax);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const lines = syntax.split('\n');

  return (
    <div className={`rounded border border-line bg-[#0c0d10] overflow-hidden my-6 ${className}`}>
      {/* Header bar */}
      <div className="px-4 py-2.5 bg-[#14161a] border-b border-line flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Terminal className="w-3.5 h-3.5 text-amber" />
          <span className="font-mono text-xs font-medium text-amber">{title}</span>
        </div>

        <button
          type="button"
          onClick={handleCopy}
          className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded bg-bg-raised text-xs font-mono text-text-muted hover:text-white border border-line transition-colors"
        >
          {copied ? (
            <>
              <Check className="w-3.5 h-3.5 text-emerald" />
              <span className="text-emerald">Copied</span>
            </>
          ) : (
            <>
              <Copy className="w-3.5 h-3.5" />
              <span>Copy</span>
            </>
          )}
        </button>
      </div>

      {/* Code body with line numbers */}
      <div className="p-4 font-mono text-xs sm:text-sm text-text leading-relaxed overflow-x-auto">
        <table className="w-full border-collapse">
          <tbody>
            {lines.map((line, idx) => (
              <tr key={idx} className="hover:bg-bg-raised/30 transition-colors">
                <td className="w-8 pr-4 text-right text-text-muted/50 select-none font-mono text-xs">
                  {idx + 1}
                </td>
                <td className="text-blue-300 font-mono">
                  {line}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
