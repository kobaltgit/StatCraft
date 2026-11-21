import React from 'react';
import { MathView } from '../math-view';

interface MarkdownRendererProps {
  content: string;
  className?: string;
}

export function MarkdownRenderer({ content, className = '' }: MarkdownRendererProps) {
  // Split into paragraphs / blocks by double newlines
  const blocks = content.split(/\n\n+/);

  return (
    <div className={`space-y-5 text-sm sm:text-base text-text-muted leading-relaxed ${className}`}>
      {blocks.map((block, idx) => {
        const trimmed = block.trim();

        // Display Math Block $$ ... $$
        if (trimmed.startsWith('$$') && trimmed.endsWith('$$')) {
          const math = trimmed.slice(2, -2).trim();
          return (
            <div key={idx} className="my-6 p-4 rounded bg-bg-sunken/80 border border-line text-center overflow-x-auto">
              <MathView math={math} block={true} />
            </div>
          );
        }

        // Heading 3: ### Title
        if (trimmed.startsWith('### ')) {
          const rawTitle = trimmed.replace('### ', '').trim();
          // generate slug id for toc linking
          const slugId = rawTitle
            .toLowerCase()
            .replace(/[^\w\s-]/g, '')
            .replace(/\s+/g, '-');

          return (
            <h3
              key={idx}
              id={slugId}
              className="text-lg sm:text-xl font-bold text-white pt-6 pb-2 border-b border-line flex items-center gap-2 group scroll-mt-24"
            >
              <span>{rawTitle}</span>
              <a
                href={`#${slugId}`}
                className="opacity-0 group-hover:opacity-100 text-text-muted hover:text-accent font-mono text-sm transition-opacity"
              >
                #
              </a>
            </h3>
          );
        }

        // Blockquote: > Text
        if (trimmed.startsWith('> ')) {
          const text = trimmed.replace(/^>\s*/gm, '');
          return (
            <blockquote
              key={idx}
              className="p-4 rounded-r border-l-2 border-accent bg-bg-raised/40 text-text text-sm sm:text-base italic leading-relaxed my-4"
            >
              {renderInlineText(text)}
            </blockquote>
          );
        }

        // Bulleted list: lines starting with *
        if (trimmed.startsWith('* ') || trimmed.startsWith('- ')) {
          const items = trimmed.split('\n').filter(l => l.trim().startsWith('* ') || l.trim().startsWith('- '));
          return (
            <ul key={idx} className="space-y-2 list-none pl-1 my-3">
              {items.map((item, itemIdx) => {
                const itemText = item.replace(/^[\*\-]\s+/, '');
                return (
                  <li key={itemIdx} className="flex items-start gap-2.5">
                    <span className="w-1.5 h-1.5 rounded-full bg-accent shrink-0 mt-2" />
                    <span>{renderInlineText(itemText)}</span>
                  </li>
                );
              })}
            </ul>
          );
        }

        // Regular paragraph
        return (
          <p key={idx} className="leading-relaxed">
            {renderInlineText(trimmed)}
          </p>
        );
      })}
    </div>
  );
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
        <code key={i} className="font-mono text-xs px-1.5 py-0.5 rounded bg-bg-sunken border border-line text-blue-300">
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
