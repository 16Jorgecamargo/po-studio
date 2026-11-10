import { useCallback, useEffect, useRef, useState } from 'react';

export function usePopover<T extends HTMLElement = HTMLDivElement>() {
  const [open, setOpen] = useState(false);
  const wrapRef = useRef<T>(null);
  const triggerRef = useRef<HTMLButtonElement>(null);

  const close = useCallback((restore = true) => {
    setOpen(false);
    if (restore) triggerRef.current?.focus();
  }, []);

  useEffect(() => {
    if (!open) return;
    const onDown = (e: PointerEvent) => {
      if (wrapRef.current && !wrapRef.current.contains(e.target as Node)) setOpen(false);
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.stopPropagation();
        close();
      }
    };
    document.addEventListener('pointerdown', onDown);
    document.addEventListener('keydown', onKey);
    return () => {
      document.removeEventListener('pointerdown', onDown);
      document.removeEventListener('keydown', onKey);
    };
  }, [open, close]);

  return { open, setOpen, close, wrapRef, triggerRef };
}

export function menuKeys(e: React.KeyboardEvent<HTMLElement>) {
  const items = Array.from(e.currentTarget.querySelectorAll<HTMLElement>('[data-menu-item]'));
  const i = items.indexOf(document.activeElement as HTMLElement);
  if (e.key === 'ArrowDown') {
    e.preventDefault();
    items[(i + 1) % items.length]?.focus();
  } else if (e.key === 'ArrowUp') {
    e.preventDefault();
    items[(i - 1 + items.length) % items.length]?.focus();
  } else if (e.key === 'Home') {
    e.preventDefault();
    items[0]?.focus();
  } else if (e.key === 'End') {
    e.preventDefault();
    items[items.length - 1]?.focus();
  }
}
