import { useEffect, useMemo, useRef, useState } from 'react';
import * as engine from '../engine';
import { navigate } from '../router';
import { useStudio } from '../store';
import type { ModuleKey } from '../types';
import { ChevronIcon } from '../ui/icons';
import { mod } from '../ui/platform';
import { usePopover } from '../ui/usePopover';

function normalize(s: string) {
  return s
    .normalize('NFD')
    .replace(/[̀-ͯ]/g, '')
    .toLowerCase();
}

export function ModuleSwitcher({ current }: { current: ModuleKey | null }) {
  const { open, setOpen, close, wrapRef, triggerRef } = usePopover();
  const [query, setQuery] = useState('');
  const [active, setActive] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const modules = engine.catalog().modules;
  const info = modules.find((m) => m.key === current);

  const filtered = useMemo(() => {
    const q = normalize(query.trim());
    if (!q) return modules;
    return modules.filter((m) => normalize(`${m.name} ${m.group} ${m.blurb}`).includes(q));
  }, [modules, query]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const t = e.target as HTMLElement;
      const typing = t.tagName === 'INPUT' || t.tagName === 'TEXTAREA' || t.tagName === 'SELECT';
      if ((e.key === 'k' || e.key === 'K') && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        setOpen(true);
      } else if (e.key === '/' && !typing) {
        e.preventDefault();
        setOpen(true);
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [setOpen]);

  useEffect(() => {
    if (open) {
      setQuery('');
      setActive(Math.max(0, modules.findIndex((m) => m.key === current)));
    }
  }, [open, current, modules]);

  const go = (key: ModuleKey) => {
    useStudio.getState().ensure(key);
    close(false);
    navigate(`/app/${key}`);
  };

  const onKey = (e: React.KeyboardEvent) => {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      setActive((a) => Math.min(filtered.length - 1, a + 1));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setActive((a) => Math.max(0, a - 1));
    } else if (e.key === 'Enter' && filtered[active]) {
      e.preventDefault();
      go(filtered[active].key);
    }
  };

  let lastGroup = '';
  return (
    <div className="switcher" ref={wrapRef}>
      <button ref={triggerRef} type="button" className="switcher-btn" aria-haspopup="dialog" aria-expanded={open} onClick={() => setOpen(!open)}>
        <span className="switcher-group">{info ? info.group : 'Módulos'}</span>
        <span className="switcher-name">{info ? info.name : 'Escolha um módulo'}</span>
        <ChevronIcon />
      </button>
      {open ? (
        <div className="popover switcher-pop" role="dialog" aria-label="Trocar de módulo">
          <div className="switcher-search">
            <input
              ref={inputRef}
              autoFocus
              className="text-input"
              placeholder="Buscar módulo"
              value={query}
              aria-label="Buscar módulo"
              aria-controls="switcher-list"
              aria-activedescendant={filtered[active] ? `sw-${filtered[active].key}` : undefined}
              role="combobox"
              aria-expanded="true"
              onChange={(e) => {
                setQuery(e.target.value);
                setActive(0);
              }}
              onKeyDown={onKey}
            />
            <kbd>{mod('K')}</kbd>
          </div>
          <ul id="switcher-list" className="switcher-list" role="listbox" aria-label="Módulos">
            {filtered.length === 0 ? <li className="switcher-empty">Nenhum módulo com esse nome.</li> : null}
            {filtered.map((m, i) => {
              const header = m.group !== lastGroup ? m.group : null;
              lastGroup = m.group;
              return (
                <li key={m.key} role="presentation">
                  {header ? <p className="switcher-head" aria-hidden="true">{header}</p> : null}
                  <div
                    id={`sw-${m.key}`}
                    role="option"
                    aria-selected={i === active}
                    className={`switcher-item${i === active ? ' is-active' : ''}${m.key === current ? ' is-current' : ''}`}
                    onPointerMove={() => setActive(i)}
                    onClick={() => go(m.key)}
                  >
                    <span className="switcher-item-name">{m.name}</span>
                    <span className="switcher-item-blurb">{m.blurb}</span>
                  </div>
                </li>
              );
            })}
          </ul>
        </div>
      ) : null}
    </div>
  );
}
