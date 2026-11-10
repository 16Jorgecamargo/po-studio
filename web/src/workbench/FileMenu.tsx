import { useEffect } from 'react';
import { useStudio } from '../store';
import type { ModuleKey } from '../types';
import { ChevronIcon } from '../ui/icons';
import { mod } from '../ui/platform';
import { menuKeys, usePopover } from '../ui/usePopover';
import { exportReport, forget, openFileDialog, save, saveAs } from './files';

export function FileMenu({ module }: { module: ModuleKey }) {
  const { open, setOpen, close, wrapRef, triggerRef } = usePopover();
  const st = useStudio.getState;

  useEffect(() => {
    if (open) window.setTimeout(() => wrapRef.current?.querySelector<HTMLElement>('[data-menu-item]')?.focus(), 0);
  }, [open, wrapRef]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (!(e.metaKey || e.ctrlKey)) return;
      const k = e.key.toLowerCase();
      if (k === 's') {
        e.preventDefault();
        if (e.shiftKey) saveAs(module);
        else save(module);
      } else if (k === 'o') {
        e.preventDefault();
        openFileDialog();
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [module]);

  const run = (fn: () => void) => () => {
    close();
    fn();
  };

  const items: ({ label: string; kbd?: string; act: () => void } | 'sep')[] = [
    {
      label: 'Novo problema',
      act: () => {
        forget(module);
        st().newProblem(module);
      },
    },
    { label: 'Carregar exemplo', act: () => st().loadExample(module) },
    'sep',
    { label: 'Abrir…', kbd: mod('O'), act: () => openFileDialog() },
    { label: 'Salvar', kbd: mod('S'), act: () => save(module) },
    { label: 'Salvar como…', kbd: mod('S', true), act: () => saveAs(module) },
    { label: 'Salvar no formato do programa original (.txt)', act: () => saveAs(module, true) },
    'sep',
    { label: 'Exportar relatório (.txt)', act: () => exportReport(module) },
  ];

  return (
    <div className="menu-wrap" ref={wrapRef}>
      <button ref={triggerRef} type="button" className="btn btn--quiet bar-btn" aria-haspopup="menu" aria-expanded={open} onClick={() => setOpen(!open)}>
        Arquivo
        <ChevronIcon />
      </button>
      {open ? (
        <div className="popover menu-pop" role="menu" aria-label="Arquivo" onKeyDown={menuKeys}>
          {items.map((it, i) =>
            it === 'sep' ? (
              <div key={i} className="menu-sep" role="separator" />
            ) : (
              <button key={i} type="button" role="menuitem" className="menu-item" data-menu-item onClick={run(it.act)}>
                <span>{it.label}</span>
                {it.kbd ? <kbd>{it.kbd}</kbd> : null}
              </button>
            ),
          )}
        </div>
      ) : null}
    </div>
  );
}
