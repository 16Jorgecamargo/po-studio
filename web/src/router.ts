import { useSyncExternalStore } from 'react';

const EVENT = 'po-studio:navigate';

function subscribe(cb: () => void) {
  window.addEventListener('popstate', cb);
  window.addEventListener(EVENT, cb);
  return () => {
    window.removeEventListener('popstate', cb);
    window.removeEventListener(EVENT, cb);
  };
}

function snapshot() {
  return window.location.pathname;
}

export function usePath(): string {
  return useSyncExternalStore(subscribe, snapshot, snapshot);
}

export function navigate(to: string, replace = false): void {
  if (to === window.location.pathname) return;
  if (replace) window.history.replaceState(null, '', to);
  else window.history.pushState(null, '', to);
  window.dispatchEvent(new Event(EVENT));
  window.scrollTo(0, 0);
}

export function onLinkClick(e: React.MouseEvent<HTMLAnchorElement>, to: string): void {
  if (e.defaultPrevented || e.button !== 0 || e.metaKey || e.ctrlKey || e.shiftKey || e.altKey) return;
  e.preventDefault();
  navigate(to);
}
