import { useMemo, useState } from 'react';
import * as engine from '../../engine';
import type { ModuleKey, NetInput, OutcomeView } from '../../types';
import { TipBox, useWidth, type Tip } from './plot';

interface Props {
  module: ModuleKey;
  input: NetInput;
  start: number;
  target: number;
  view: OutcomeView | null;
}

export function NetworkView({ module, input, start, target, view }: Props) {
  const res = useMemo(() => engine.networkArcs(input), [input]);
  const [ref, width] = useWidth<HTMLDivElement>();
  const [tip, setTip] = useState<Tip | null>(null);
  if (!res.ok) return <div className="error-note"><strong>Atenção:</strong> {res.error}</div>;

  const { names, arcs } = res.value;
  const n = names.length;
  const directed = module !== 'minimal_spanning_tree';
  const highlight = view?.highlight ?? [];
  const height = Math.max(360, Math.min(600, width * 0.75));
  const cx = width / 2;
  const cy = height / 2;
  const rad = Math.max(70, Math.min(width, height) / 2 - 52);
  const pos = Array.from({ length: n }, (_, i) => {
    const a = (Math.PI * 2 * i) / n - Math.PI / 2;
    return { x: cx + Math.cos(a) * rad, y: cy + Math.sin(a) * rad };
  });
  const has = (i: number, j: number) => arcs.some((a) => a.from === i && a.to === j);
  const shown = arcs.filter((a) => directed || !(a.to < a.from && has(a.to, a.from)));
  const special = (i: number) => (module === 'minimal_spanning_tree' ? i === start : i === start || i === target);
  const key = JSON.stringify([input.matrix, highlight]);
  const nodeR = n > 14 ? 15 : 19;

  return (
    <div className="graph-view">
      <p className="hint">{highlight.length ? 'Arcos da solução em azul, na ordem em que o algoritmo os escolheu.' : 'Rede informada. Resolva para destacar a solução.'}</p>
      <div className="chart" ref={ref}>
        <svg key={key} width={width} height={height} role="img" aria-label={`Rede com ${n} nós e ${shown.length} ${directed ? 'arcos' : 'ligações'}`}>
          <defs>
            <marker id="arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
              <path d="M0 0 L10 5 L0 10 z" className="arrow-head" />
            </marker>
            <marker id="arrow-lit" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="5" markerHeight="5" orient="auto-start-reverse">
              <path d="M0 0 L10 5 L0 10 z" className="arrow-head arrow-head--lit" />
            </marker>
          </defs>
          {shown.map((a, idx) => {
            const k = highlight.findIndex((h) => (h.from === a.from && h.to === a.to) || (!directed && h.from === a.to && h.to === a.from));
            const lit = k >= 0;
            const reversed = lit && highlight[k].from === a.to;
            const p = pos[a.from];
            const q = pos[a.to];
            const dx = q.x - p.x;
            const dy = q.y - p.y;
            const len = Math.hypot(dx, dy) || 1;
            const ux = dx / len;
            const uy = dy / len;
            const bidir = directed && has(a.to, a.from);
            const ox = bidir ? -uy * 6 : 0;
            const oy = bidir ? ux * 6 : 0;
            const x1 = p.x + ux * (nodeR + 2) + ox;
            const y1 = p.y + uy * (nodeR + 2) + oy;
            const x2 = q.x - ux * (nodeR + 4) + ox;
            const y2 = q.y - uy * (nodeR + 4) + oy;
            const mx = (x1 + x2) / 2 + ox * 1.6;
            const my = (y1 + y2) / 2 + oy * 1.6;
            const label = lit && module === 'maximal_flow' ? `${highlight[k].label}/${a.label}` : a.label;
            const tipText = `${names[a.from]} ${directed ? '→' : '–'} ${names[a.to]}: ${label}`;
            return (
              <g key={idx}>
                <line className="arc" x1={x1} y1={y1} x2={x2} y2={y2} markerEnd={directed && !lit ? 'url(#arrow)' : undefined} />
                {lit ? (
                  <line
                    className="arc arc--lit draw-line"
                    pathLength={1}
                    x1={reversed ? x2 : x1}
                    y1={reversed ? y2 : y1}
                    x2={reversed ? x1 : x2}
                    y2={reversed ? y1 : y2}
                    markerEnd={directed && !reversed ? 'url(#arrow-lit)' : undefined}
                    markerStart={directed && reversed ? 'url(#arrow-lit)' : undefined}
                    style={{ animationDelay: `${300 + k * 220}ms` }}
                  />
                ) : null}
                <g
                  className={`arc-label${lit ? ' is-lit' : ''}`}
                  tabIndex={0}
                  aria-label={tipText}
                  onPointerEnter={() => setTip({ x: mx, y: my, content: tipText })}
                  onPointerLeave={() => setTip(null)}
                  onFocus={() => setTip({ x: mx, y: my, content: tipText })}
                  onBlur={() => setTip(null)}
                >
                  <rect x={mx - (label.length * 7.4 + 10) / 2} y={my - 10} width={label.length * 7.4 + 10} height={20} rx={4} />
                  <text x={mx} y={my + 4.5} textAnchor="middle">
                    {label}
                  </text>
                </g>
              </g>
            );
          })}
          {pos.map((p, i) => (
            <g key={i} className={`net-node${special(i) ? ' is-special' : ''}`}>
              <circle cx={p.x} cy={p.y} r={nodeR} />
              <text x={p.x} y={p.y + 4.5} textAnchor="middle">
                {names[i].length > 4 ? names[i].slice(0, 4) : names[i]}
              </text>
            </g>
          ))}
        </svg>
        <TipBox tip={tip} width={width} />
      </div>
      <p className="hint">
        {module === 'minimal_spanning_tree'
          ? 'Nó preenchido: nó inicial.'
          : module === 'maximal_flow'
            ? 'Nós preenchidos: origem e destino. Nos arcos da solução, o rótulo mostra fluxo/capacidade.'
            : 'Nós preenchidos: origem e destino.'}
      </p>
    </div>
  );
}
