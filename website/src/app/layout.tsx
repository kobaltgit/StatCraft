import type { Metadata } from 'next';
import './globals.css';

export const metadata: Metadata = {
  metadataBase: new URL('https://statcraft.dev'),
  title: 'StatCraft // Open-Science Statistical Computing in Safe Rust',
  description: 'An experimental, 100% Safe Rust re-implementation of classical statistical procedures. Verified against SciPy Golden Oracle (Δ ≤ 10⁻⁶) with native Model Context Protocol (MCP) bridge.',
  icons: {
    icon: '/favicon.ico',
    apple: '/icon-256.png',
  },
  openGraph: {
    title: 'StatCraft // Open-Science Statistical Computing',
    description: '100% Safe Rust, Apache Arrow & Polars engine, verified against SciPy Golden Oracle to 10⁻⁶ precision.',
    images: ['/github-preview.png'],
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="dark scroll-smooth">
      <head>
        <link
          rel="stylesheet"
          href="https://cdn.jsdelivr.net/npm/katex@0.16.11/dist/katex.min.css"
          integrity="sha384-nB0miv6/jRmo5UMMR1wu3Gz6NLsoNdZdWCOE9LX/4zG9uv563ksr53DEhbiS3/fo"
          crossOrigin="anonymous"
        />
      </head>
      <body className="min-h-screen bg-bg text-text antialiased font-sans selection:bg-accent/30 selection:text-white">
        {children}
      </body>
    </html>
  );
}
