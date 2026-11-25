import React from 'react';
import { MathView } from '../math-view';

interface MarkdownRendererProps {
  content: string;
  className?: string;
  toc?: { id: string }[];
}

export function slugifyHeading(title: string): string {
  return title
    .toLowerCase()
    .replace(/[^\p{L}\p{N}\s-]/gu, '')
    .trim()
    .replace(/\s+/g, '-')
    .replace(/-+/g, '-');
}

export function MarkdownRenderer({ content, className = '', toc }: MarkdownRendererProps) {
  // First, extract display math blocks $$...$$ to avoid splitting them
  const rawSections = content.split(/(\$\$[\s\S]*?\$\$)/g);
  let h3Index = 0;

  const elements: React.ReactNode[] = [];

  rawSections.forEach((section, sIdx) => {
    const trimmed = section.trim();
    if (!trimmed) return;

    // Display Math: $$ ... $$
    if (trimmed.startsWith('$$') && trimmed.endsWith('$$')) {
      const math = trimmed.slice(2, -2).trim();
      elements.push(
        <div
          key={`math-${sIdx}`}
          className="my-5 p-4 rounded bg-bg-sunken border border-line text-center overflow-x-auto text-base text-white"
        >
          <MathView math={math} block={true} />
        </div>
      );
      return;
    }

    // Process line-by-line for headings, lists, blockquotes, paragraphs
    const lines = section.split('\n');
    let currentParagraph: string[] = [];
    let currentList: { type: 'ul' | 'ol'; items: string[] } | null = null;

    const flushParagraph = () => {
      if (currentParagraph.length > 0) {
        const text = currentParagraph.join(' ').trim();
        if (text) {
          elements.push(
            <p key={`p-${sIdx}-${elements.length}`} className="text-sm sm:text-[15px] text-text-muted leading-relaxed font-normal my-3">
              {renderInlineText(text)}
            </p>
          );
        }
        currentParagraph = [];
      }
    };

    const flushList = () => {
      if (currentList && currentList.items.length > 0) {
        if (currentList.type === 'ol') {
          elements.push(
            <ol key={`ol-${sIdx}-${elements.length}`} className="space-y-2 my-3 pl-1">
              {currentList.items.map((item, i) => (
                <li key={i} className="flex items-start gap-2 text-sm sm:text-[15px] text-text-muted leading-relaxed">
                  <span className="font-mono text-xs text-blue-400 font-semibold shrink-0 mt-0.5 w-5">
                    {i + 1}.
                  </span>
                  <span>{renderInlineText(item)}</span>
                </li>
              ))}
            </ol>
          );
        } else {
          elements.push(
            <ul key={`ul-${sIdx}-${elements.length}`} className="space-y-2 my-3 pl-1">
              {currentList.items.map((item, i) => (
                <li key={i} className="flex items-start gap-2.5 text-sm sm:text-[15px] text-text-muted leading-relaxed">
                  <span className="w-1.5 h-1.5 rounded-full bg-accent shrink-0 mt-2" />
                  <span>{renderInlineText(item)}</span>
                </li>
              ))}
            </ul>
          );
        }
        currentList = null;
      }
    };

    lines.forEach((line) => {
      const lineTrim = line.trim();

      // Empty line -> flush current structures
      if (!lineTrim) {
        flushParagraph();
        flushList();
        return;
      }

      // Heading 3: ### Title
      if (lineTrim.startsWith('### ')) {
        flushParagraph();
        flushList();
        const title = lineTrim.replace(/^###\s+/, '').trim();
        const slugId = toc?.[h3Index]?.id || slugifyHeading(title);
        h3Index += 1;

        elements.push(
          <h3
            key={`h3-${sIdx}-${elements.length}`}
            id={slugId}
            className="text-base sm:text-lg font-bold text-white pt-6 pb-2 border-b border-line flex items-center gap-2 group scroll-mt-20 mt-4 mb-2"
          >
            <span>{title}</span>
            <a
              href={`#${slugId}`}
              className="opacity-0 group-hover:opacity-100 text-text-muted hover:text-accent font-mono text-xs transition-opacity"
            >
              #
            </a>
          </h3>
        );
        return;
      }

      // Blockquote: > Text
      if (lineTrim.startsWith('> ')) {
        flushParagraph();
        flushList();
        const text = lineTrim.replace(/^>\s*/, '');
        elements.push(
          <blockquote
            key={`bq-${sIdx}-${elements.length}`}
            className="p-3.5 my-3 rounded-r border-l-2 border-accent bg-bg-raised/40 text-text text-sm sm:text-[15px] italic leading-relaxed"
          >
            {renderInlineText(text)}
          </blockquote>
        );
        return;
      }

      // Unordered list: * or -
      if (lineTrim.startsWith('* ') || lineTrim.startsWith('- ')) {
        flushParagraph();
        const itemText = lineTrim.replace(/^[\*\-]\s+/, '');
        if (!currentList || currentList.type !== 'ul') {
          flushList();
          currentList = { type: 'ul', items: [itemText] };
        } else {
          currentList.items.push(itemText);
        }
        return;
      }

      // Numbered list: 1. , 2.
      const olMatch = lineTrim.match(/^\d+\.\s+(.*)$/);
      if (olMatch) {
        flushParagraph();
        const itemText = olMatch[1];
        if (!currentList || currentList.type !== 'ol') {
          flushList();
          currentList = { type: 'ol', items: [itemText] };
        } else {
          currentList.items.push(itemText);
        }
        return;
      }

      // Otherwise regular text line
      if (currentList) {
        flushList();
      }
      currentParagraph.push(lineTrim);
    });

    flushParagraph();
    flushList();
  });

  return <div className={`space-y-1 ${className}`}>{elements}</div>;
}

function renderInlineText(text: string): React.ReactNode {
  // Regex to match $...$, `...`, **...**
  const regex = /(\$[^$]+\$|`[^`]+`|\*\*[^*]+\*\*)/g;
  const parts = text.split(regex);

  return parts.map((part, i) => {
    if (!part) return null;

    // Inline Math: $...$
    if (part.startsWith('$') && part.endsWith('$') && part.length > 2) {
      const math = part.slice(1, -1);
      return <MathView key={i} math={math} block={false} />;
    }

    // Inline Code: `...`
    if (part.startsWith('`') && part.endsWith('`') && part.length > 2) {
      const code = part.slice(1, -1);
      return (
        <code key={i} className="font-mono text-xs px-1.5 py-0.5 rounded bg-bg-sunken border border-line text-blue-300 font-normal">
          {code}
        </code>
      );
    }

    // Bold text: **...**
    if (part.startsWith('**') && part.endsWith('**') && part.length > 4) {
      const bold = part.slice(2, -2);
      return <strong key={i} className="text-white font-semibold">{bold}</strong>;
    }

    return part;
  });
}
