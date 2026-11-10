import { useMemo, useState } from 'react';
import * as engine from '../../engine';
import type { LpInput } from '../../types';
import { penLoop } from '../../ui/pen';
import { Axes, Legend, makeFrame, series, seriesDash, sx, sy, TipBox, useWidth, type Tip } from './plot';

interface Props {
  input: LpInput;
  hidden: number[];
  onHidden: (h: number[]) => void;
  showPath: boolean;
  onShowPath: (v: boolean) => void;
}

export function LpGraphView({ input, hidden, onHidden, showPath, onShowPath }: Props) {
  const graph = useMemo(() => engine.lpGraph(input), [input]);
  const path = useMemo(() => (graph.ok ? engine.lpPath(input) : null), [graph, input]);
  const [ref, width] = useWidth<HTMLDivElement>();
  const [tip, setTip] = useState<Tip | null>(null);

  if (!graph.ok) {
    return graph.info ? <p className="notice">{graph.error}</p> : <div className="error-note"><strong>Atenção:</strong> {graph.error}</div>;
  }
  const g = graph.value;
  const height = Math.max(340, Math.min(560, width * 0.72));
  const f = makeFrame(width, height, 0, g.xmax, 0, g.ymax, width < 480);
  const key = JSON.stringify([input.obj, input.a, input.rhs, input.rel, input.maximize, g.xmax, g.ymax]);
  const toggle = (i: number) => onHidden(hidden.includes(i) ? hidden.filter((k) => k !== i) : [...hidden, i]);
  const region = g.region.map(([x, y]) => `${sx(f, x)},${sy(f, y)}`).join(' ');

  let sweep: { d: string; dx: number; dy: number } | null = null;
  if (g.optimum && g.z !== null) {
    const [c1, c2] = g.c;
    const cc = c1 * c1 + c2 * c2;
    if (cc > 1e-12) {
      const [ox, oy] = g.optimum;
      const span = 4 * (g.xmax + g.ymax);
      const dn = Math.sqrt(cc);
      const ax = ox - (c2 / dn) * span;
      const ay = oy + (c1 / dn) * span;
      const bx = ox + (c2 / dn) * span;
      const by = oy - (c1 / dn) * span;
      const k = g.z / cc;
      sweep = {
        d: `M ${sx(f, ax)} ${sy(f, ay)} L ${sx(f, bx)} ${sy(f, by)}`,
        dx: sx(f, ox - k * c1) - sx(f, ox),
        dy: sy(f, oy - k * c2) - sy(f, oy),
      };
    }
  }
  const lineDelay = (i: number) => `${i * 90}ms`;
  const afterLines = g.lines.length * 90 + 420;
  const pts = path && path.ok ? path.value.points.filter((p) => p.x.length >= 2) : [];
  const clipId = `lp-clip-${Math.round(width)}`;

  return (
    <div className="graph-view">
      <div className="graph-head">
        <p className="graph-summary">{g.summary}</p>
        <label className="toggle-inline">
          <input type="checkbox" checked={showPath} onChange={(e) => onShowPath(e.target.checked)} />
          Mostrar o caminho do simplex
        </label>
      </div>
      <Legend
        items={g.lines.map((l, i) => ({
          label: l.label,
          color: series(i),
          off: hidden.includes(i),
          onToggle: () => toggle(i),
        }))}
      />
      <div className="chart" ref={ref} onPointerLeave={() => setTip(null)}>
        <svg key={key} width={width} height={height} role="img" aria-label={`Gráfico de ${g.xLabel} por ${g.yLabel}. ${g.summary}`}>
          <defs>
            <clipPath id={clipId}>
              <rect x={f.left} y={f.top} width={f.right - f.left} height={f.bottom - f.top} />
            </clipPath>
          </defs>
          <Axes f={f} xLabel={g.xLabel} yLabel={g.yLabel} />
          {g.region.length >= 3 ? <polygon className="region draw-fade" points={region} style={{ animationDelay: `${afterLines - 300}ms` }} /> : null}
          <g clipPath={`url(#${clipId})`}>
            {g.lines.map((l, i) =>
              l.segment && !hidden.includes(i) ? (
                <g key={i}>
                  <line
                    className={`series-line${seriesDash(i) ? "" : " draw-line"}`}
                    pathLength={1}
                    x1={sx(f, l.segment[0][0])}
                    y1={sy(f, l.segment[0][1])}
                    x2={sx(f, l.segment[1][0])}
                    y2={sy(f, l.segment[1][1])}
                    stroke={series(i)}
                    strokeDasharray={seriesDash(i)}
                    style={{ animationDelay: lineDelay(i) }}
                  />
                </g>
              ) : null,
            )}
            {sweep ? (
              <path
                className="objective-line"
                d={sweep.d}
                style={{ ['--dx' as string]: `${sweep.dx}px`, ['--dy' as string]: `${sweep.dy}px`, animationDelay: `${afterLines}ms` }}
              />
            ) : null}
          </g>
          {g.lines.map((l, i) =>
            l.segment && !hidden.includes(i) ? (
              <text key={`t${i}`} className="direct-label" x={sx(f, l.segment[0][0]) + 5} y={sy(f, l.segment[0][1]) - 6}>
                {l.tag}
              </text>
            ) : null,
          )}
          {showPath && pts.length > 1 ? (
            <g className="simplex-path">
              <polyline points={pts.map((p) => `${sx(f, p.x[0])},${sy(f, p.x[1])}`).join(' ')} />
              {pts.map((p, i) => (
                <g key={i}>
                  <circle
                    className="path-dot"
                    cx={sx(f, p.x[0])}
                    cy={sy(f, p.x[1])}
                    r={8.5}
                    tabIndex={0}
                    aria-label={`Iteração ${i}: ${g.xLabel} = ${engine.formatNum(p.x[0])}, ${g.yLabel} = ${engine.formatNum(p.x[1])}, z = ${engine.formatNum(p.z)}`}
                    onPointerEnter={() => setTip({ x: sx(f, p.x[0]), y: sy(f, p.x[1]), content: <>Iteração {i}<br />({engine.formatNum(p.x[0])}, {engine.formatNum(p.x[1])})<br />z = {engine.formatNum(p.z)}</> })}
                    onFocus={() => setTip({ x: sx(f, p.x[0]), y: sy(f, p.x[1]), content: <>Iteração {i}<br />({engine.formatNum(p.x[0])}, {engine.formatNum(p.x[1])})<br />z = {engine.formatNum(p.z)}</> })}
                    onBlur={() => setTip(null)}
                  />
                  <text className="path-num" x={sx(f, p.x[0])} y={sy(f, p.x[1]) + 3.8} textAnchor="middle" aria-hidden="true">
                    {i}
                  </text>
                </g>
              ))}
            </g>
          ) : null}
          {g.region.map(([x, y], i) => (
            <circle
              key={`v${i}`}
              className="vertex-hit"
              cx={sx(f, x)}
              cy={sy(f, y)}
              r={9}
              onPointerEnter={() => setTip({ x: sx(f, x), y: sy(f, y), content: <>Vértice ({engine.formatNum(x)}, {engine.formatNum(y)})<br />z = {engine.formatNum(g.c[0] * x + g.c[1] * y)}</> })}
            />
          ))}
          {g.optimum ? (
            <g className="optimum" style={{ animationDelay: `${afterLines + 700}ms` }}>
              <circle className="optimum-dot" cx={sx(f, g.optimum[0])} cy={sy(f, g.optimum[1])} r={6} />
              <path className="pen-stroke draw-line" pathLength={1} d={penLoop(key, sx(f, g.optimum[0]), sy(f, g.optimum[1]), 17, 15)} style={{ animationDelay: `${afterLines + 700}ms` }} />
              <text className="optimum-label" x={sx(f, g.optimum[0]) + (sx(f, g.optimum[0]) > f.right - 160 ? -22 : 22)} y={sy(f, g.optimum[1]) - 16} textAnchor={sx(f, g.optimum[0]) > f.right - 160 ? 'end' : 'start'}>
                {g.marker}
              </text>
            </g>
          ) : null}
        </svg>
        <TipBox tip={tip} width={width} />
      </div>
      <p className="hint">
        Região viável em azul claro. A reta tracejada é a função objetivo deslizando até o ótimo. Passe o mouse nos vértices para ver as coordenadas e o valor de z.
      </p>
    </div>
  );
}
