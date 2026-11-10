import { useEffect, useState } from 'react';
import * as engine from '../engine';
import type { ModuleKey } from '../types';
import { openDropped } from './files';
import { ProblemPane } from './ProblemPane';
import { SolutionPane } from './SolutionPane';
import { TopBar } from './TopBar';
import { ModuleIndex } from '../landing/ModuleIndex';

function useNarrow() {
  const query = '(max-width: 1099px)';
  const [narrow, setNarrow] = useState(() => window.matchMedia(query).matches);
  useEffect(() => {
    const m = window.matchMedia(query);
    const on = () => setNarrow(m.matches);
    m.addEventListener('change', on);
    return () => m.removeEventListener('change', on);
  }, []);
  return narrow;
}

export function Workbench({ module }: { module: ModuleKey | null }) {
  const narrow = useNarrow();
  const [pane, setPane] = useState<'problem' | 'solution'>('problem');
  const [dragging, setDragging] = useState(false);

  useEffect(() => {
    const info = engine.catalog().modules.find((m) => m.key === module);
    document.title = info ? `${info.name} · PO Studio` : 'Módulos · PO Studio';
  }, [module]);

  useEffect(() => setPane('problem'), [module]);

  useEffect(() => {
    if (narrow) window.scrollTo(0, 0);
  }, [pane, narrow]);

  if (!module) {
    return (
      <div className="app">
        <a className="skip-link" href="#main">
          Pular para o conteúdo
        </a>
        <TopBar module={null} />
        <main id="main" className="desk desk--index">
          <div className="sheet index-sheet">
            <h1>Escolha um módulo</h1>
            <p className="sheet-blurb">Cada módulo abre com um exemplo pronto. Troque os números e clique em Resolver.</p>
            <ModuleIndex />
          </div>
        </main>
      </div>
    );
  }

  return (
    <div
      className="app"
      onDragOver={(e) => {
        if (e.dataTransfer.types.includes('Files')) {
          e.preventDefault();
          setDragging(true);
        }
      }}
      onDragLeave={(e) => {
        if (e.currentTarget === e.target) setDragging(false);
      }}
      onDrop={(e) => {
        const file = e.dataTransfer.files?.[0];
        setDragging(false);
        if (file) {
          e.preventDefault();
          openDropped(file);
        }
      }}
    >
      <a className="skip-link" href="#main">
        Pular para o enunciado
      </a>
      <TopBar module={module} />
      {narrow ? (
        <div className="pane-switch" role="tablist" aria-label="Mostrar">
          <button type="button" role="tab" aria-selected={pane === 'problem'} className={pane === 'problem' ? 'is-on' : undefined} onClick={() => setPane('problem')}>
            Enunciado
          </button>
          <button type="button" role="tab" aria-selected={pane === 'solution'} className={pane === 'solution' ? 'is-on' : undefined} onClick={() => setPane('solution')}>
            Solução
          </button>
        </div>
      ) : null}
      <main id="main" className={`desk desk--work${narrow ? ` show-${pane}` : ''}`}>
        <ProblemPane module={module} onSolved={() => narrow && setPane('solution')} />
        <SolutionPane module={module} />
      </main>
      {dragging ? (
        <div className="drop-hint" aria-hidden="true">
          <p>Solte o arquivo .json ou .txt para abrir</p>
        </div>
      ) : null}
    </div>
  );
}
