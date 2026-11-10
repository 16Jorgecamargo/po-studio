import type { BbNodeView } from '../../types';
import { penLoop } from '../../ui/pen';

interface Props {
  nodes: BbNodeView[];
  best: number | null;
  selected: number | null;
  onSelect: (id: number) => void;
}

const DX = 118;
const DY = 112;

function layout(nodes: BbNodeView[]) {
  const xs = new Map<number, number>();
  const kids = new Map<number, number[]>();
  nodes.forEach((n) => {
    if (n.parent !== null) kids.set(n.parent, [...(kids.get(n.parent) ?? []), n.id]);
  });
  let leaf = 0;
  const place = (id: number) => {
    const ks = kids.get(id) ?? [];
    if (ks.length === 0) {
      xs.set(id, leaf);
      leaf += 1;
    } else {
      ks.forEach(place);
      xs.set(id, ks.reduce((s, k) => s + (xs.get(k) ?? 0), 0) / ks.length);
    }
  };
  const roots = nodes.filter((n) => n.parent === null);
  roots.forEach((r) => place(r.id));
  return { xs, leaves: Math.max(leaf, 1) };
}

function kind(n: BbNodeView, best: number | null) {
  if (n.id === best) return 'best';
  return n.status;
}

const LEGEND: { k: string; label: string }[] = [
  { k: 'best', label: 'Melhor solução' },
  { k: 'integer', label: 'Inteira' },
  { k: 'branched', label: 'Ramificado' },
  { k: 'fathomed', label: 'Sondado (z pior)' },
  { k: 'infeasible', label: 'Inviável' },
];

export function BbTreeView({ nodes, best, selected, onSelect }: Props) {
  if (nodes.length > 400) {
    return <p className="notice">Árvore grande demais para desenhar. Veja os subproblemas na aba Subproblemas.</p>;
  }
  const { xs, leaves } = layout(nodes);
  const depth = Math.max(0, ...nodes.map((n) => n.depth));
  const width = leaves * DX + 40;
  const height = (depth + 1) * DY + 50;
  const pos = (n: BbNodeView) => ({ x: 20 + DX / 2 + (xs.get(n.id) ?? 0) * DX, y: 36 + n.depth * DY });
  const byId = new Map(nodes.map((n) => [n.id, n]));
  const step = Math.min(80, 1300 / Math.max(nodes.length, 1));
  const sel = selected !== null ? byId.get(selected) : undefined;

  return (
    <div className="graph-view">
      <ul className="legend bb-legend">
        {LEGEND.map((l) => (
          <li key={l.k} className="legend-item">
            <svg width="18" height="18" aria-hidden="true">
              <g className={`bb-node k-${l.k}`}>
                <circle cx="9" cy="9" r="7" />
              </g>
            </svg>
            {l.label}
          </li>
        ))}
        <li className="hint">Clique em um nó para ver a solução.</li>
      </ul>
      <div className="tree-scroll" tabIndex={-1}>
        <svg width={width} height={height} role="group" aria-label="Árvore do Branch & Bound">
          {nodes.map((n, i) => {
            if (n.parent === null) return null;
            const p = byId.get(n.parent);
            if (!p) return null;
            const a0 = pos(p);
            const a = { x: a0.x, y: a0.y + (p.zText ? 44 : 23) };
            const b0 = pos(n);
            const b = { x: b0.x, y: b0.y - 23 };
            const lx = a.x + (b.x - a.x) * 0.74;
            const ly = a.y + (b.y - a.y) * 0.74;
            return (
              <g key={`e${n.id}`}>
                <line className="tree-edge draw-line" pathLength={1} x1={a.x} y1={a.y} x2={b.x} y2={b.y} style={{ animationDelay: `${Math.max(0, i * step - 120)}ms` }} />
                <rect className="edge-label-bg" x={lx - (n.branch.length * 6.9 + 8) / 2} y={ly - 9} width={n.branch.length * 6.9 + 8} height={18} rx={3} />
                <text className="edge-label" x={lx} y={ly + 4} textAnchor="middle">
                  {n.branch}
                </text>
              </g>
            );
          })}
          {nodes.map((n, i) => {
            const p = pos(n);
            const k = kind(n, best);
            const label = `Nó N${n.id}: ${n.statusText}${n.zText ? `, z = ${n.zText}` : ''}`;
            return (
              <g
                key={n.id}
                className={`bb-node k-${k}${selected === n.id ? ' is-selected' : ''} pop-in`}
                style={{ animationDelay: `${i * step}ms`, transformOrigin: `${p.x}px ${p.y}px` }}
                role="button"
                tabIndex={0}
                aria-pressed={selected === n.id}
                aria-label={label}
                onClick={() => onSelect(n.id)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter' || e.key === ' ') {
                    e.preventDefault();
                    onSelect(n.id);
                  }
                }}
              >
                <circle cx={p.x} cy={p.y} r={21} />
                {k === 'infeasible' || k === 'unbounded' ? <line className="strike" x1={p.x - 13} y1={p.y + 13} x2={p.x + 13} y2={p.y - 13} /> : null}
                <text className="bb-id" x={p.x} y={p.y + 4.5} textAnchor="middle">
                  N{n.id}
                </text>
                {n.zText ? (
                  <text className="bb-z" x={p.x} y={p.y + 37} textAnchor="middle">
                    z={n.zText}
                  </text>
                ) : null}
                {k === 'best' ? <path className="pen-stroke draw-line" pathLength={1} d={penLoop(`bb${n.id}`, p.x, p.y, 29, 27)} style={{ animationDelay: `${nodes.length * step + 200}ms` }} /> : null}
              </g>
            );
          })}
        </svg>
      </div>
      {sel ? (
        <div className="node-detail" aria-live="polite">
          <h4>Subproblema N{sel.id}</h4>
          <p>Ramos que o definem: {sel.pathText}</p>
          <p>Situação: {sel.statusText}</p>
          {sel.zText ? <p>Valor objetivo: {sel.zText}</p> : null}
          {sel.solutionText ? <p>Solução: {sel.solutionText}</p> : null}
        </div>
      ) : null}
    </div>
  );
}
