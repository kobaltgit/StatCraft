import React from 'react';
import katex from 'katex';

interface MathViewProps {
  math: string;
  block?: boolean;
  className?: string;
}

export function MathView({ math, block = false, className = '' }: MathViewProps) {
  let html = '';
  try {
    html = katex.renderToString(math, {
      displayMode: block,
      throwOnError: false,
      output: 'html',
    });
  } catch {
    html = `<span class="text-rose-400 font-mono text-xs">${math}</span>`;
  }

  return (
    <span
      className={`inline-block max-w-full ${className}`.trim()}
      dangerouslySetInnerHTML={{ __html: html }}
    />
  );
}
