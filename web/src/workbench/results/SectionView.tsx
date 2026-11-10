import { useState } from 'react';
import type { Cell, Section } from '../../types';
import { PenCircle } from '../../ui/PenCircle';
import { CopyIcon } from '../../ui/icons';

function looksNumeric(text: string): boolean {
  const ch = text.charAt(0);
  return ch !== '' && (/[0-9]/.test(ch) || '-−+([∞.–'.includes(ch));
}

function numericColumns(s: Section, ncol: number): boolean[] {
  return Array.from({ length: ncol }, (_, j) => {
    if (j === 0) return false;
    const filled = s.rows.map((r) => r[j]).filter((c): c is Cell => !!c && c.text !== '');
    if (filled.length === 0) return false;
    const hits = filled.filter((c) => looksNumeric(c.text)).length;
    return hits * 2 > filled.length;
  });
}

function tableText(s: Section): string {
  const lines: string[] = [];
  if (s.headers.length) lines.push(s.headers.join('\t'));
  s.rows.forEach((r) => lines.push(r.map((c) => c.text).join('\t')));
  return lines.join('\n') + '\n';
}

function CellContent({ cell, seed }: { cell: Cell; seed: string }) {
  if (cell.style === 'pivot') {
    return (
      <span className="pivot-mark">
        {cell.text}
        <PenCircle seed={seed} />
        <span className="sr-only"> (pivô)</span>
      </span>
    );
  }
  return <>{cell.text}</>;
}

interface Props {
  section: Section;
  id: string;
  fresh?: boolean;
}

export function SectionView({ section: s, id, fresh }: Props) {
  const [copied, setCopied] = useState(false);
  const ncol = Math.max(s.headers.length, ...s.rows.map((r) => r.length), 0);
  const numeric = numericColumns(s, ncol);
  const hasTable = s.headers.length > 0 || s.rows.length > 0;

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(tableText(s));
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1600);
    } catch {
      setCopied(false);
    }
  };

  return (
    <section className={`ws-section${fresh ? ' is-fresh' : ''}`} aria-labelledby={`${id}-t`}>
      <header className="ws-section-head">
        <div>
          <h3 id={`${id}-t`}>{s.title || 'Quadro'}</h3>
          {s.subtitle ? <p className="ws-sub">{s.subtitle}</p> : null}
        </div>
        {hasTable ? (
          <button type="button" className="btn btn--quiet btn--small copy-btn" onClick={copy} aria-label={`Copiar tabela: ${s.title}`}>
            <CopyIcon />
            <span aria-live="polite">{copied ? 'Copiada' : 'Copiar'}</span>
          </button>
        ) : null}
      </header>
      {s.lines.map((l, i) => (
        <p key={i} className="ws-line">
          {l}
        </p>
      ))}
      {hasTable ? (
        <div className="table-wrap" tabIndex={0} role="region" aria-label={`Tabela: ${s.title}`}>
          <table className="tableau">
            {s.headers.length ? (
              <thead>
                <tr>
                  {Array.from({ length: ncol }, (_, j) => (
                    <th key={j} scope="col" className={numeric[j] ? 'num' : undefined}>
                      {s.headers[j] ?? ''}
                    </th>
                  ))}
                </tr>
              </thead>
            ) : null}
            <tbody>
              {s.rows.map((r, i) => (
                <tr key={i} className={r.some((c) => c.style === 'red') ? 'row-leaving' : undefined}>
                  {Array.from({ length: ncol }, (_, j) => {
                    const cell = r[j] ?? { text: '', style: 'normal' as const };
                    const cls = [`c-${cell.style}`, numeric[j] ? 'num' : ''].filter(Boolean).join(' ');
                    if (j === 0) {
                      return (
                        <th key={j} scope="row" className={cls}>
                          <CellContent cell={cell} seed={`${id}-${i}-${j}`} />
                        </th>
                      );
                    }
                    return (
                      <td key={j} className={cls}>
                        <CellContent cell={cell} seed={`${id}-${i}-${j}`} />
                      </td>
                    );
                  })}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : null}
      {s.notes.map((n, i) => (
        <p key={i} className="ws-note">
          {n}
        </p>
      ))}
    </section>
  );
}

export function SectionList({ sections, id }: { sections: Section[]; id: string }) {
  return (
    <div className="section-list">
      {sections.map((s, i) => (
        <SectionView key={`${id}-${i}`} section={s} id={`${id}-${i}`} />
      ))}
    </div>
  );
}
