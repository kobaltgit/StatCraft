'use client';

import { useEffect } from 'react';
import { usePathname, useRouter } from 'next/navigation';

export function LocaleDetector() {
  const pathname = usePathname();
  const router = useRouter();

  useEffect(() => {
    // Only perform client check on root path if no language cookie is set
    if (pathname === '/') {
      const hasLangCookie = document.cookie.split(';').some((c) => c.trim().startsWith('statcraft_lang='));
      if (!hasLangCookie) {
        const userLang = (navigator.languages && navigator.languages[0]) || navigator.language || '';
        if (userLang.toLowerCase().startsWith('ru')) {
          document.cookie = 'statcraft_lang=ru; path=/; max-age=31536000; SameSite=Lax';
          router.replace('/ru');
        }
      }
    }
  }, [pathname, router]);

  return null;
}
