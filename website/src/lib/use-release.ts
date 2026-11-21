'use client';

import { useState, useEffect } from 'react';
import {
  FALLBACK_RELEASE,
  detectUserOs,
  getPrimaryAsset,
  type ReleaseData,
  type ReleaseAsset,
  type UserOs,
  type UserArch,
} from './release';

export interface UseReleaseResult {
  release: ReleaseData;
  version: string;
  tagName: string;
  userOs: UserOs;
  userArch: UserArch;
  osLabelEn: string;
  osLabelRu: string;
  primaryAsset: ReleaseAsset;
  allAssets: ReleaseAsset[];
  isLoading: boolean;
}

export function useRelease(): UseReleaseResult {
  const [release, setRelease] = useState<ReleaseData>(FALLBACK_RELEASE);
  const [osInfo, setOsInfo] = useState<{
    os: UserOs;
    arch: UserArch;
    labelEn: string;
    labelRu: string;
  }>({
    os: 'windows',
    arch: 'x64',
    labelEn: 'Windows',
    labelRu: 'Windows',
  });
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    // 1. Detect client OS and Arch
    const detected = detectUserOs();
    setOsInfo(detected);

    // 2. Fetch latest release from API route (with local caching)
    let isMounted = true;
    setIsLoading(true);

    fetch('/api/release')
      .then((res) => {
        if (!res.ok) throw new Error('Failed to load release info');
        return res.json();
      })
      .then((data: ReleaseData) => {
        if (isMounted && data && data.version) {
          setRelease(data);
        }
      })
      .catch((err) => {
        // Quiet fallback to FALLBACK_RELEASE
        console.warn('Using fallback release info:', err);
      })
      .finally(() => {
        if (isMounted) setIsLoading(false);
      });

    return () => {
      isMounted = false;
    };
  }, []);

  const primaryAsset = getPrimaryAsset(release, osInfo.os, osInfo.arch);

  return {
    release,
    version: release.version,
    tagName: release.tagName,
    userOs: osInfo.os,
    userArch: osInfo.arch,
    osLabelEn: osInfo.labelEn,
    osLabelRu: osInfo.labelRu,
    primaryAsset,
    allAssets: release.assets || FALLBACK_RELEASE.assets,
    isLoading,
  };
}
