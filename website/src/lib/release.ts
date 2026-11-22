export type UserOs = 'windows' | 'macos' | 'linux' | 'unknown';
export type UserArch = 'x64' | 'arm64' | 'unknown';

export interface ReleaseAsset {
  name: string;
  downloadUrl: string;
  size?: number;
  os: 'windows' | 'macos' | 'linux';
  kind: 'installer' | 'portable' | 'deb' | 'dmg';
  arch: 'x64' | 'arm64';
  labelEn: string;
  labelRu: string;
  badge: string;
  recommended?: boolean;
}

export interface ReleaseData {
  tagName: string;
  version: string;
  publishedAt?: string;
  htmlUrl: string;
  assets: ReleaseAsset[];
}

export const GITHUB_REPO = 'kobaltgit/StatCraft';
export const DEFAULT_VERSION = '0.4.5';
export const DEFAULT_TAG = `v${DEFAULT_VERSION}`;

/**
 * Constructs direct GitHub asset download URL based on release tag and filename
 */
export function getAssetDownloadUrl(tagName: string, assetName: string): string {
  return `https://github.com/${GITHUB_REPO}/releases/download/${tagName}/${assetName}`;
}

/**
 * Builds the standard matrix of release assets conforming to .github/workflows/release.yml
 */
export function buildStandardAssets(tagName: string): ReleaseAsset[] {
  return [
    {
      name: 'statcraft-windows-x64-installer.exe',
      downloadUrl: getAssetDownloadUrl(tagName, 'statcraft-windows-x64-installer.exe'),
      os: 'windows',
      kind: 'installer',
      arch: 'x64',
      labelEn: 'Windows Installer (Setup .exe)',
      labelRu: 'Установщик Windows (Setup .exe)',
      badge: '.exe',
      recommended: true,
    },
    {
      name: 'statcraft-windows-x64.zip',
      downloadUrl: getAssetDownloadUrl(tagName, 'statcraft-windows-x64.zip'),
      os: 'windows',
      kind: 'portable',
      arch: 'x64',
      labelEn: 'Windows Portable (.zip)',
      labelRu: 'Портативная сборка Windows (.zip)',
      badge: '.zip',
    },
    {
      name: 'statcraft-macos-arm64.dmg',
      downloadUrl: getAssetDownloadUrl(tagName, 'statcraft-macos-arm64.dmg'),
      os: 'macos',
      kind: 'dmg',
      arch: 'arm64',
      labelEn: 'macOS Apple Silicon (M1/M2/M3/M4 .dmg)',
      labelRu: 'macOS Apple Silicon (M1/M2/M3/M4 .dmg)',
      badge: '.dmg',
      recommended: true,
    },
    {
      name: 'statcraft-macos-arm64.tar.gz',
      downloadUrl: getAssetDownloadUrl(tagName, 'statcraft-macos-arm64.tar.gz'),
      os: 'macos',
      kind: 'portable',
      arch: 'arm64',
      labelEn: 'macOS Apple Silicon Portable (.tar.gz)',
      labelRu: 'macOS Apple Silicon Portable (.tar.gz)',
      badge: '.tar.gz',
    },
    {
      name: 'statcraft-macos-x64.dmg',
      downloadUrl: getAssetDownloadUrl(tagName, 'statcraft-macos-x64.dmg'),
      os: 'macos',
      kind: 'dmg',
      arch: 'x64',
      labelEn: 'macOS Intel (.dmg)',
      labelRu: 'macOS Intel (.dmg)',
      badge: '.dmg',
    },
    {
      name: 'statcraft-macos-x64.tar.gz',
      downloadUrl: getAssetDownloadUrl(tagName, 'statcraft-macos-x64.tar.gz'),
      os: 'macos',
      kind: 'portable',
      arch: 'x64',
      labelEn: 'macOS Intel Portable (.tar.gz)',
      labelRu: 'macOS Intel Portable (.tar.gz)',
      badge: '.tar.gz',
    },
    {
      name: 'statcraft-linux-x64.deb',
      downloadUrl: getAssetDownloadUrl(tagName, 'statcraft-linux-x64.deb'),
      os: 'linux',
      kind: 'deb',
      arch: 'x64',
      labelEn: 'Linux Debian / Ubuntu (.deb)',
      labelRu: 'Linux Debian / Ubuntu (.deb)',
      badge: '.deb',
      recommended: true,
    },
    {
      name: 'statcraft-linux-x64.tar.gz',
      downloadUrl: getAssetDownloadUrl(tagName, 'statcraft-linux-x64.tar.gz'),
      os: 'linux',
      kind: 'portable',
      arch: 'x64',
      labelEn: 'Linux Portable (.tar.gz)',
      labelRu: 'Linux Портативный архив (.tar.gz)',
      badge: '.tar.gz',
    },
  ];
}

export const FALLBACK_RELEASE: ReleaseData = {
  tagName: DEFAULT_TAG,
  version: DEFAULT_VERSION,
  publishedAt: '2026-11-20T21:45:00Z',
  htmlUrl: `https://github.com/${GITHUB_REPO}/releases/tag/${DEFAULT_TAG}`,
  assets: buildStandardAssets(DEFAULT_TAG),
};

/**
 * Client-side user OS and architecture detector
 */
export function detectUserOs(): { os: UserOs; arch: UserArch; labelEn: string; labelRu: string } {
  if (typeof window === 'undefined') {
    return { os: 'windows', arch: 'x64', labelEn: 'Windows (x64)', labelRu: 'Windows (x64)' };
  }

  const ua = window.navigator.userAgent.toLowerCase();
  const platform = ((window.navigator as unknown as { userAgentData?: { platform?: string } }).userAgentData?.platform || window.navigator.platform || '').toLowerCase();

  // 1. Windows detection
  if (platform.includes('win') || ua.includes('windows') || ua.includes('win32') || ua.includes('win64')) {
    return { os: 'windows', arch: 'x64', labelEn: 'Windows', labelRu: 'Windows' };
  }

  // 2. macOS detection
  if (platform.includes('mac') || ua.includes('macintosh') || ua.includes('mac os x')) {
    // Distinguish Apple Silicon (ARM64) vs Intel (x64)
    // In modern macOS, Apple Silicon is standard. If ua explicitly indicates Intel x86_64 and not arm:
    const isIntel = ua.includes('intel') && !ua.includes('arm');
    const arch: UserArch = isIntel ? 'x64' : 'arm64';
    const archLabel = arch === 'arm64' ? 'Apple Silicon' : 'Intel';
    return {
      os: 'macos',
      arch,
      labelEn: `macOS (${archLabel})`,
      labelRu: `macOS (${archLabel})`,
    };
  }

  // 3. Linux detection
  if (platform.includes('linux') || (ua.includes('linux') && !ua.includes('android'))) {
    return { os: 'linux', arch: 'x64', labelEn: 'Linux', labelRu: 'Linux' };
  }

  return { os: 'unknown', arch: 'x64', labelEn: 'Other OS', labelRu: 'Другие ОС' };
}

/**
 * Returns primary recommended asset for the given user OS and architecture
 */
export function getPrimaryAsset(release: ReleaseData, os: UserOs, arch?: UserArch): ReleaseAsset {
  const assets = release.assets && release.assets.length > 0 ? release.assets : FALLBACK_RELEASE.assets;

  if (os === 'windows') {
    // Prefer installer .exe
    return assets.find((a) => a.os === 'windows' && a.kind === 'installer') || assets[0];
  }

  if (os === 'macos') {
    const targetArch = arch === 'x64' ? 'x64' : 'arm64';
    // Match dmg with matching arch, fallback to arm64 dmg
    return (
      assets.find((a) => a.os === 'macos' && a.kind === 'dmg' && a.arch === targetArch) ||
      assets.find((a) => a.os === 'macos' && a.kind === 'dmg') ||
      assets[0]
    );
  }

  if (os === 'linux') {
    // Prefer .deb installer
    return (
      assets.find((a) => a.os === 'linux' && a.kind === 'deb') ||
      assets.find((a) => a.os === 'linux') ||
      assets[0]
    );
  }

  // Default fallback to Windows installer
  return assets.find((a) => a.os === 'windows' && a.kind === 'installer') || assets[0];
}
