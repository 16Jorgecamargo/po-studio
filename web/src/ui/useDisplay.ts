import { useEffect, useSyncExternalStore } from 'react';
import { useStudio } from '../store';

function media(query: string) {
  return {
    subscribe(cb: () => void) {
      const m = window.matchMedia(query);
      m.addEventListener('change', cb);
      return () => m.removeEventListener('change', cb);
    },
    get() {
      return window.matchMedia(query).matches;
    },
  };
}

const darkQuery = media('(prefers-color-scheme: dark)');
const motionQuery = media('(prefers-reduced-motion: reduce)');

export function useSystemDark(): boolean {
  return useSyncExternalStore(darkQuery.subscribe, darkQuery.get, () => false);
}

export function useSystemReducedMotion(): boolean {
  return useSyncExternalStore(motionQuery.subscribe, motionQuery.get, () => false);
}

export function useEffectiveTheme(): 'light' | 'dark' {
  const theme = useStudio((s) => s.theme);
  const systemDark = useSystemDark();
  return theme ?? (systemDark ? 'dark' : 'light');
}

export function useReducedMotion(): boolean {
  const pref = useStudio((s) => s.reduceMotion);
  const system = useSystemReducedMotion();
  return pref ?? system;
}

export function useApplyDisplay(): void {
  const theme = useStudio((s) => s.theme);
  const reduced = useReducedMotion();
  useEffect(() => {
    const root = document.documentElement;
    if (theme) root.dataset.theme = theme;
    else delete root.dataset.theme;
  }, [theme]);
  useEffect(() => {
    document.documentElement.dataset.motion = reduced ? 'reduced' : 'full';
  }, [reduced]);
}
