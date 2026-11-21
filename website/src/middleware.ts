import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';

/**
 * Checks if Russian is preferred in Accept-Language header
 */
function isRussianPreferred(acceptLang: string): boolean {
  if (!acceptLang) return false;

  // Split and parse language tags with quality values e.g. "ru-RU,ru;q=0.9,en-US;q=0.8,en;q=0.7"
  const entries = acceptLang.split(',').map((entry) => {
    const parts = entry.trim().split(';');
    const langCode = parts[0].toLowerCase().trim();
    let q = 1.0;
    if (parts[1]) {
      const qMatch = parts[1].match(/q=([\d.]+)/);
      if (qMatch) {
        q = parseFloat(qMatch[1]);
      }
    }
    return { lang: langCode, q: isNaN(q) ? 1.0 : q };
  });

  entries.sort((a, b) => b.q - a.q);

  if (entries.length === 0) return false;

  // If primary/first preference is Russian
  if (entries[0].lang.startsWith('ru')) {
    return true;
  }

  // Find ru entry vs en entry
  const ruEntry = entries.find((e) => e.lang.startsWith('ru'));
  const enEntry = entries.find((e) => e.lang.startsWith('en'));

  if (ruEntry && (!enEntry || ruEntry.q >= enEntry.q)) {
    return true;
  }

  return false;
}

export function middleware(request: NextRequest) {
  const { pathname, search } = request.nextUrl;

  // Ignore static assets, next internal files, and API endpoints
  if (
    pathname.startsWith('/_next') ||
    pathname.startsWith('/api') ||
    pathname.includes('.') ||
    pathname.startsWith('/favicon')
  ) {
    return NextResponse.next();
  }

  // 1. Check for manual override in query string: ?lang=ru or ?lang=en
  const queryLang = request.nextUrl.searchParams.get('lang');
  if (queryLang === 'ru' || queryLang === 'en') {
    const targetUrl = request.nextUrl.clone();
    targetUrl.searchParams.delete('lang');
    if (queryLang === 'ru' && !pathname.startsWith('/ru')) {
      targetUrl.pathname = `/ru${pathname === '/' ? '' : pathname}`;
    } else if (queryLang === 'en' && pathname.startsWith('/ru')) {
      targetUrl.pathname = pathname.replace(/^\/ru(\/|$)/, '$1') || '/';
    }
    const res = NextResponse.redirect(targetUrl);
    res.cookies.set('statcraft_lang', queryLang, {
      path: '/',
      maxAge: 60 * 60 * 24 * 365,
      sameSite: 'lax',
    });
    return res;
  }

  const cookieLang = request.cookies.get('statcraft_lang')?.value;

  // 2. Routing for Root '/'
  if (pathname === '/') {
    // If user has an explicit preference saved in cookie
    if (cookieLang === 'ru') {
      return NextResponse.redirect(new URL(`/ru${search}`, request.url));
    }
    if (cookieLang === 'en') {
      return NextResponse.next();
    }

    // Auto-detect from Accept-Language header
    const acceptLanguage = request.headers.get('accept-language') || '';
    if (isRussianPreferred(acceptLanguage)) {
      const redirectRes = NextResponse.redirect(new URL(`/ru${search}`, request.url));
      redirectRes.cookies.set('statcraft_lang', 'ru', {
        path: '/',
        maxAge: 60 * 60 * 24 * 365,
        sameSite: 'lax',
      });
      return redirectRes;
    } else {
      const nextRes = NextResponse.next();
      nextRes.cookies.set('statcraft_lang', 'en', {
        path: '/',
        maxAge: 60 * 60 * 24 * 365,
        sameSite: 'lax',
      });
      return nextRes;
    }
  }

  // 3. Routing for '/docs'
  if (pathname === '/docs') {
    if (cookieLang === 'ru') {
      return NextResponse.redirect(new URL(`/ru/docs${search}`, request.url));
    }
    if (!cookieLang) {
      const acceptLanguage = request.headers.get('accept-language') || '';
      if (isRussianPreferred(acceptLanguage)) {
        const redirectRes = NextResponse.redirect(new URL(`/ru/docs${search}`, request.url));
        redirectRes.cookies.set('statcraft_lang', 'ru', {
          path: '/',
          maxAge: 60 * 60 * 24 * 365,
          sameSite: 'lax',
        });
        return redirectRes;
      }
    }
  }

  // 4. Synchronize cookie with active language prefix
  if (pathname.startsWith('/ru')) {
    const res = NextResponse.next();
    if (cookieLang !== 'ru') {
      res.cookies.set('statcraft_lang', 'ru', {
        path: '/',
        maxAge: 60 * 60 * 24 * 365,
        sameSite: 'lax',
      });
    }
    return res;
  } else {
    const res = NextResponse.next();
    if (cookieLang !== 'en') {
      res.cookies.set('statcraft_lang', 'en', {
        path: '/',
        maxAge: 60 * 60 * 24 * 365,
        sameSite: 'lax',
      });
    }
    return res;
  }
}

export const config = {
  matcher: ['/', '/docs', '/docs/:path*', '/ru', '/ru/:path*'],
};
