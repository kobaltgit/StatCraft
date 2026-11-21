import { NextResponse } from 'next/server';
import {
  FALLBACK_RELEASE,
  GITHUB_REPO,
  buildStandardAssets,
  type ReleaseData,
  type ReleaseAsset,
} from '@/lib/release';

interface GitHubReleaseAsset {
  name: string;
  browser_download_url: string;
  size: number;
}

interface GitHubReleaseResponse {
  tag_name: string;
  name: string;
  published_at: string;
  html_url: string;
  assets: GitHubReleaseAsset[];
}

export async function GET() {
  try {
    const res = await fetch(`https://api.github.com/repos/${GITHUB_REPO}/releases/latest`, {
      headers: {
        Accept: 'application/vnd.github+json',
        'User-Agent': 'StatCraft-Website-ReleaseDetector',
      },
      next: { revalidate: 1800 }, // 30 minutes ISR
    });

    if (!res.ok) {
      return NextResponse.json(FALLBACK_RELEASE, {
        status: 200,
        headers: {
          'Cache-Control': 'public, s-maxage=600, stale-while-revalidate=3600',
        },
      });
    }

    const data: GitHubReleaseResponse = await res.json();
    const tagName = data.tag_name || FALLBACK_RELEASE.tagName;
    const version = tagName.replace(/^v/, '');

    // Match raw assets with standard metadata or enrich standard assets with actual URLs/sizes
    const standardAssets = buildStandardAssets(tagName);
    const enrichedAssets: ReleaseAsset[] = standardAssets.map((std) => {
      const remote = data.assets?.find((a) => a.name.toLowerCase() === std.name.toLowerCase());
      if (remote) {
        return {
          ...std,
          downloadUrl: remote.browser_download_url,
          size: remote.size,
        };
      }
      return std;
    });

    const releaseData: ReleaseData = {
      tagName,
      version,
      publishedAt: data.published_at || FALLBACK_RELEASE.publishedAt,
      htmlUrl: data.html_url || `https://github.com/${GITHUB_REPO}/releases/tag/${tagName}`,
      assets: enrichedAssets,
    };

    return NextResponse.json(releaseData, {
      status: 200,
      headers: {
        'Cache-Control': 'public, s-maxage=1800, stale-while-revalidate=86400',
      },
    });
  } catch (err) {
    console.error('Error fetching latest release from GitHub API:', err);
    return NextResponse.json(FALLBACK_RELEASE, {
      status: 200,
      headers: {
        'Cache-Control': 'public, s-maxage=600, stale-while-revalidate=3600',
      },
    });
  }
}
