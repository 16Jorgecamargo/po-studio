import { useEffect, useRef } from 'react';
import type { Section } from '../../types';
import { SectionView } from './SectionView';

interface Props {
  id: string;
  sections: Section[];
  shown: number;
  onShown: (k: number) => void;
  legend?: string;
  unit?: string;
}

export function IterationPlayer({ id, sections, shown, onShown, legend, unit = 'Quadro' }: Props) {
  const total = sections.length;
  const k = Math.min(Math.max(shown, 1), total);
  const lastRef = useRef<HTMLDivElement>(null);
  const prevK = useRef(k);

  useEffect(() => {
    const el = lastRef.current;
    if (k > prevK.current && el) {
      const behavior = document.documentElement.dataset.motion === 'reduced' ? 'auto' : 'smooth';
      const box = el.closest<HTMLElement>('.sheet-body');
      const scrolls = box && getComputedStyle(box).overflowY === 'auto' && box.scrollHeight > box.clientHeight;
      if (box && scrolls) {
        const top = el.getBoundingClientRect().top - box.getBoundingClientRect().top + box.scrollTop - 150;
        box.scrollTo({ top, behavior });
      } else {
        const top = el.getBoundingClientRect().top + window.scrollY - 200;
        window.scrollTo({ top, behavior });
      }
    }
    prevK.current = k;
  }, [k]);

  if (total === 0) return null;

  const onKey = (e: React.KeyboardEvent) => {
    if (e.key === 'ArrowRight') {
      e.preventDefault();
      onShown(Math.min(total, k + 1));
    } else if (e.key === 'ArrowLeft') {
      e.preventDefault();
      onShown(Math.max(1, k - 1));
    } else if (e.key === 'Home') {
      e.preventDefault();
      onShown(1);
    } else if (e.key === 'End') {
      e.preventDefault();
      onShown(total);
    }
  };

  return (
    <div className="player">
      <div className="player-bar" role="group" aria-label="Controle das iterações" onKeyDown={onKey}>
        <div className="player-buttons">
          <button type="button" className="btn btn--primary btn--small" onClick={() => onShown(Math.min(total, k + 1))} disabled={k >= total}>
            Próxima iteração
          </button>
          <button type="button" className="btn btn--small" onClick={() => onShown(Math.max(1, k - 1))} disabled={k <= 1}>
            Anterior
          </button>
          <button type="button" className="btn btn--small" onClick={() => onShown(total)} disabled={k >= total}>
            Todas as iterações
          </button>
          <button type="button" className="btn btn--quiet btn--small" onClick={() => onShown(1)} disabled={k <= 1}>
            Reiniciar
          </button>
        </div>
        <p className="player-count" aria-live="polite">
          Mostrando {k} de {total}
        </p>
        {total > 1 && total <= 60 ? (
          <ol className="player-ticks" aria-label="Ir para">
            {sections.map((s, i) => (
              <li key={i}>
                <button
                  type="button"
                  className={`step-tick${i < k ? ' is-done' : ''}${i === k - 1 ? ' is-current' : ''}`}
                  onClick={() => onShown(i + 1)}
                  aria-label={`${unit} ${i + 1}: ${s.title}`}
                  aria-current={i === k - 1 ? 'step' : undefined}
                  title={s.title}
                >
                  {i + 1}
                </button>
              </li>
            ))}
          </ol>
        ) : null}
      </div>
      {legend ? <p className="hint player-legend">{legend}</p> : null}
      <div className="section-list">
        {sections.slice(0, k).map((s, i) => (
          <div key={`${id}-${i}`} ref={i === k - 1 ? lastRef : undefined}>
            <SectionView section={s} id={`${id}-${i}`} fresh={i === k - 1 && k > 1} />
          </div>
        ))}
      </div>
    </div>
  );
}
