'use client';

import React, { useState, useRef, useEffect } from 'react';
import { Download, ChevronDown, Check, ExternalLink } from 'lucide-react';
import { useRelease } from '@/lib/use-release';
import { type ReleaseAsset } from '@/lib/release';

interface DownloadButtonProps {
  lang?: 'en' | 'ru';
  variant?: 'hero' | 'nav' | 'mobile';
  className?: string;
}

export function DownloadButton({
  lang = 'en',
  variant = 'hero',
  className = '',
}: DownloadButtonProps) {
  const { version, tagName, userOs, osLabelEn, osLabelRu, primaryAsset, allAssets } = useRelease();
  const [dropdownOpen, setDropdownOpen] = useState(false);
  const [downloadStarted, setDownloadStarted] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  const isRu = lang === 'ru';
  const osLabel = isRu ? osLabelRu : osLabelEn;

  // Close dropdown on click outside
  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setDropdownOpen(false);
      }
    }
    if (dropdownOpen) {
      document.addEventListener('mousedown', handleClickOutside);
    }
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [dropdownOpen]);

  const handleDownloadClick = (asset: ReleaseAsset) => {
    setDownloadStarted(true);
    setDropdownOpen(false);
    setTimeout(() => setDownloadStarted(false), 3000);
  };

  // Group assets by OS
  const windowsAssets = allAssets.filter((a) => a.os === 'windows');
  const macAssets = allAssets.filter((a) => a.os === 'macos');
  const linuxAssets = allAssets.filter((a) => a.os === 'linux');

  // OS Icons
  const renderOsIcon = (os: string, className = 'w-4 h-4') => {
    if (os === 'windows') {
      return (
        <svg className={className} viewBox="0 0 24 24" fill="currentColor">
          <path d="M0 3.449L9.75 2.1v9.451H0m10.949-9.602L24 0v11.4H10.949M0 12.6h9.75v9.451L0 20.699M10.949 12.6H24V24l-13.051-1.801" />
        </svg>
      );
    }
    if (os === 'macos') {
      return (
        <svg className={className} viewBox="0 0 24 24" fill="currentColor">
          <path d="M18.71 19.5c-.83 1.24-1.71 2.45-3.05 2.47-1.34.03-1.77-.79-3.29-.79-1.53 0-2 .77-3.27.82-1.31.05-2.3-1.32-3.14-2.53C4.25 17 2.94 12.45 4.7 9.39c.87-1.52 2.43-2.48 4.12-2.51 1.28-.02 2.5.87 3.29.87.78 0 2.26-1.07 3.81-.91.65.03 2.47.26 3.64 1.98-.09.06-2.17 1.28-2.15 3.81.03 3.02 2.65 4.03 2.68 4.04-.03.07-.42 1.44-1.38 2.83M15.97 6.38c.62-.75 1.04-1.8 0.93-2.85-.9.04-1.98.6-2.61 1.34-.56.64-1.05 1.71-.92 2.73 1 .08 2.02-.5 2.6-1.22z" />
        </svg>
      );
    }
    if (os === 'linux') {
      return (
        <svg className={className} viewBox="0 0 24 24" fill="currentColor">
          <path d="M12.012 0c-3.178 0-5.412 2.502-5.412 5.952 0 1.968.742 3.818 1.472 5.378-.29.412-.647.784-1.06 1.103-1.688 1.306-3.805 1.83-4.802 3.69-.645 1.203-.497 2.775.297 3.874 1.05 1.455 3.018 2.01 4.757 2.003 1.182 0 2.372-.345 3.32-.962 1.137.606 2.443.962 3.83.962 1.387 0 2.693-.356 3.83-.962.948.617 2.138.962 3.32.962 1.739.007 3.707-.548 4.757-2.003.794-1.099.942-2.671.297-3.874-.997-1.86-3.114-2.384-4.802-3.69-.413-.319-.77-.691-1.06-1.103.73-1.56 1.472-3.41 1.472-5.378C21.424 2.502 19.19 0 16.012 0h-4z" />
        </svg>
      );
    }
    return <Download className={className} />;
  };

  // Dropdown render
  const renderDropdown = (align: 'left' | 'right' = 'left') => (
    <div
      className={`absolute top-full mt-2 rounded-lg border border-line bg-bg-sunken shadow-2xl p-3 z-50 animate-in fade-in slide-in-from-top-2 duration-150 text-left ${
        align === 'left'
          ? 'left-0 right-0 sm:right-auto sm:left-0 w-full sm:w-84 max-w-full'
          : 'right-0 left-auto w-80 max-w-[calc(100vw-2rem)]'
      }`}
    >
      <div className="text-[11px] font-mono uppercase tracking-wider text-text-muted px-2 py-1 mb-1 border-b border-line flex items-center justify-between">
        <span>{isRu ? 'Все доступные платформы' : 'All Platform Binaries'}</span>
        <span className="text-accent font-semibold">{tagName}</span>
      </div>

      <div className="space-y-3 py-1">
        {/* Windows */}
        <div>
          <div className="flex items-center gap-1.5 px-2 text-xs font-semibold text-white mb-1">
            {renderOsIcon('windows', 'w-3.5 h-3.5 text-blue-400 shrink-0')}
            <span>Windows (x64)</span>
          </div>
          <div className="space-y-0.5">
            {windowsAssets.map((asset) => (
              <a
                key={asset.name}
                href={asset.downloadUrl}
                download
                onClick={() => handleDownloadClick(asset)}
                className="flex items-center justify-between px-2.5 py-1.5 rounded hover:bg-bg-raised text-xs text-text-muted hover:text-white transition-colors group cursor-pointer gap-2"
              >
                <div className="flex items-center gap-2 min-w-0">
                  <Download className="w-3.5 h-3.5 text-text-muted group-hover:text-accent transition-colors shrink-0" />
                  <span className="truncate">{isRu ? asset.labelRu : asset.labelEn}</span>
                </div>
                <span className="text-[10px] font-mono px-1.5 py-0.5 rounded bg-bg-raised border border-line text-text-muted group-hover:text-white shrink-0">
                  {asset.badge}
                </span>
              </a>
            ))}
          </div>
        </div>

        {/* macOS */}
        <div>
          <div className="flex items-center gap-1.5 px-2 text-xs font-semibold text-white mb-1">
            {renderOsIcon('macos', 'w-3.5 h-3.5 text-purple-400 shrink-0')}
            <span>macOS (ARM64 & Intel)</span>
          </div>
          <div className="space-y-0.5">
            {macAssets.map((asset) => (
              <a
                key={asset.name}
                href={asset.downloadUrl}
                download
                onClick={() => handleDownloadClick(asset)}
                className="flex items-center justify-between px-2.5 py-1.5 rounded hover:bg-bg-raised text-xs text-text-muted hover:text-white transition-colors group cursor-pointer gap-2"
              >
                <div className="flex items-center gap-2 min-w-0">
                  <Download className="w-3.5 h-3.5 text-text-muted group-hover:text-accent transition-colors shrink-0" />
                  <span className="truncate">{isRu ? asset.labelRu : asset.labelEn}</span>
                </div>
                <span className="text-[10px] font-mono px-1.5 py-0.5 rounded bg-bg-raised border border-line text-text-muted group-hover:text-white shrink-0">
                  {asset.badge}
                </span>
              </a>
            ))}
          </div>
        </div>

        {/* Linux */}
        <div>
          <div className="flex items-center gap-1.5 px-2 text-xs font-semibold text-white mb-1">
            {renderOsIcon('linux', 'w-3.5 h-3.5 text-amber shrink-0')}
            <span>Linux (x64)</span>
          </div>
          <div className="space-y-0.5">
            {linuxAssets.map((asset) => (
              <a
                key={asset.name}
                href={asset.downloadUrl}
                download
                onClick={() => handleDownloadClick(asset)}
                className="flex items-center justify-between px-2.5 py-1.5 rounded hover:bg-bg-raised text-xs text-text-muted hover:text-white transition-colors group cursor-pointer gap-2"
              >
                <div className="flex items-center gap-2 min-w-0">
                  <Download className="w-3.5 h-3.5 text-text-muted group-hover:text-accent transition-colors shrink-0" />
                  <span className="truncate">{isRu ? asset.labelRu : asset.labelEn}</span>
                </div>
                <span className="text-[10px] font-mono px-1.5 py-0.5 rounded bg-bg-raised border border-line text-text-muted group-hover:text-white shrink-0">
                  {asset.badge}
                </span>
              </a>
            ))}
          </div>
        </div>
      </div>

      {/* GitHub Releases footer link */}
      <div className="pt-2 mt-1 border-t border-line">
        <a
          href="https://github.com/kobaltgit/StatCraft/releases"
          target="_blank"
          rel="noreferrer"
          className="flex items-center justify-between px-2.5 py-1 rounded text-[11px] text-text-muted hover:text-white hover:bg-bg-raised transition-colors"
        >
          <span>{isRu ? 'Все релизы и исходники на GitHub' : 'All releases & source on GitHub'}</span>
          <ExternalLink className="w-3 h-3 shrink-0 ml-1" />
        </a>
      </div>
    </div>
  );

  // 1. Hero variant: Large split download button
  if (variant === 'hero') {
    return (
      <div ref={dropdownRef} className={`relative flex items-stretch rounded shadow-lg w-full sm:w-auto ${className}`}>
        {/* Direct Download Action */}
        <a
          href={primaryAsset.downloadUrl}
          download
          onClick={() => handleDownloadClick(primaryAsset)}
          className="flex-1 sm:flex-initial inline-flex items-center justify-center sm:justify-start gap-3 px-5 py-3 rounded-l bg-accent text-white font-medium hover:bg-accent-hover text-sm sm:text-base transition-all duration-150 shadow-sm cursor-pointer group"
          title={`${isRu ? 'Скачать прямо сейчас' : 'Direct Download'}: ${primaryAsset.name}`}
        >
          {downloadStarted ? (
            <Check className="w-5 h-5 text-emerald shrink-0 animate-bounce" />
          ) : (
            renderOsIcon(userOs, 'w-5 h-5 shrink-0 text-white/90 group-hover:scale-110 transition-transform')
          )}
          <div className="flex flex-col text-left leading-tight">
            <span className="font-semibold text-white">
              {downloadStarted
                ? isRu
                  ? 'Загрузка началась...'
                  : 'Downloading...'
                : isRu
                ? `Скачать для ${osLabel}`
                : `Download for ${osLabel}`}
            </span>
            <span className="text-[11px] text-blue-200/90 font-mono font-normal">
              {primaryAsset.badge} • {tagName}
            </span>
          </div>
        </a>

        {/* Chevron Dropdown Toggle */}
        <button
          type="button"
          onClick={() => setDropdownOpen(!dropdownOpen)}
          className="flex items-center justify-center px-3.5 sm:px-3 rounded-r bg-accent/90 hover:bg-accent-hover text-white border-l border-white/20 transition-colors cursor-pointer shrink-0"
          aria-label={isRu ? 'Выбрать другую операционную систему' : 'Select another operating system'}
        >
          <ChevronDown className={`w-4 h-4 transition-transform duration-200 ${dropdownOpen ? 'rotate-180' : ''}`} />
        </button>

        {dropdownOpen && renderDropdown('left')}
      </div>
    );
  }

  // 2. Nav variant: Compact button in top header
  if (variant === 'nav') {
    return (
      <div ref={dropdownRef} className="relative hidden md:flex items-stretch">
        <a
          href={primaryAsset.downloadUrl}
          download
          onClick={() => handleDownloadClick(primaryAsset)}
          className="flex items-center gap-2 border-l border-line bg-accent px-3.5 xl:px-4 font-mono text-xs font-semibold text-white hover:bg-accent-hover transition-colors cursor-pointer group"
          title={`${isRu ? 'Скачать' : 'Download'} ${primaryAsset.name}`}
        >
          {downloadStarted ? (
            <Check className="h-3.5 w-3.5 text-emerald animate-bounce" />
          ) : (
            renderOsIcon(userOs, 'h-3.5 w-3.5 text-white/90 group-hover:scale-110 transition-transform')
          )}
          <span>{tagName}</span>
          <span className="text-[10px] px-1 py-0.2 rounded bg-white/20 text-white font-normal">
            {primaryAsset.badge}
          </span>
        </a>

        <button
          type="button"
          onClick={() => setDropdownOpen(!dropdownOpen)}
          className="flex items-center justify-center px-2 bg-accent hover:bg-accent-hover text-white border-l border-white/20 border-r border-line transition-colors cursor-pointer"
          aria-label={isRu ? 'Выбрать ОС' : 'Select OS'}
        >
          <ChevronDown className={`h-3 w-3 transition-transform duration-200 ${dropdownOpen ? 'rotate-180' : ''}`} />
        </button>

        {dropdownOpen && renderDropdown('right')}
      </div>
    );
  }

  // 3. Mobile variant: full-width block in drawer
  return (
    <div ref={dropdownRef} className="w-full space-y-2">
      <a
        href={primaryAsset.downloadUrl}
        download
        onClick={() => handleDownloadClick(primaryAsset)}
        className="flex items-center justify-between w-full px-4 py-3 rounded bg-accent text-white font-medium text-sm hover:bg-accent-hover transition-colors cursor-pointer"
      >
        <div className="flex items-center gap-2.5">
          {renderOsIcon(userOs, 'w-4 h-4 text-white')}
          <span>
            {isRu ? `Скачать ${tagName} для ${osLabel}` : `Download ${tagName} for ${osLabel}`}
          </span>
        </div>
        <span className="text-[11px] font-mono px-2 py-0.5 rounded bg-white/20">
          {primaryAsset.badge}
        </span>
      </a>

      <button
        type="button"
        onClick={() => setDropdownOpen(!dropdownOpen)}
        className="w-full flex items-center justify-between px-3 py-2 rounded bg-bg-raised text-xs text-text-muted hover:text-white transition-colors"
      >
        <span>{isRu ? 'Другие операционные системы' : 'Other operating systems'}</span>
        <ChevronDown className={`w-3.5 h-3.5 transition-transform ${dropdownOpen ? 'rotate-180' : ''}`} />
      </button>

      {dropdownOpen && (
        <div className="p-2 rounded border border-line bg-bg-sunken space-y-2 text-xs">
          {allAssets.map((asset) => (
            <a
              key={asset.name}
              href={asset.downloadUrl}
              download
              onClick={() => handleDownloadClick(asset)}
              className="flex items-center justify-between p-2 rounded hover:bg-bg-raised text-text-muted hover:text-white"
            >
              <div className="flex items-center gap-2">
                <Download className="w-3.5 h-3.5 text-accent" />
                <span>{isRu ? asset.labelRu : asset.labelEn}</span>
              </div>
              <span className="font-mono text-[10px] text-text-muted">{asset.badge}</span>
            </a>
          ))}
        </div>
      )}
    </div>
  );
}
