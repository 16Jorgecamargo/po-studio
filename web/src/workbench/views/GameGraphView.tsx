import { useMemo, useState } from 'react';
import * as engine from '../../engine';
import type { GameInput } from '../../types';
import { penLoop } from '../../ui/pen';
import { Axes, Legend, makeFrame, series, seriesDash, sx, sy, TipBox, useWidth, type Tip } from './plot';

export function GameGraphView({ input }: { input: GameInput }) {
  const res = useMemo(() => engine.gameGraph(input), [input]);
  const [ref, width] = useWidth<HTMLDivElement>();
  const [tip, setTip] = useState<Tip | null>(null);
  const [cursor, setCursor] = useState<number | null>(null);

  if (!res.ok) {
    return res.info ? <p className="notice">{res.error}</p> : <div className="error-note"><strong>Atenção:</strong> {res.error}</div>;
  }
  const g = res.value;
  const pad = (g.ymax - g.ymin) * 0.1;
  const height = Math.max(320, Math.min(520, width * 0.66));
  const f = makeFrame(width, height, 0, 1, g.ymin - pad, g.ymax + pad, width < 480);
  const key = JSON.stringify(input.payoff);
  const linesEnd = g.lines.length * 90 + 420;
  const env = g.envelope.map(([p, v]) => `${sx(f, p)},${sy(f, v)}`).join(' ');
  const ox = sx(f, g.best_p);
  const oy = sy(f, g.best_v);

  const onMove = (e: React.PointerEvent<SVGRectElement>) => {
    const box = (e.currentTarget.ownerSVGElement as SVGSVGElement).getBoundingClientRect();
    const px = e.clientX - box.left;
    const p = Math.min(1, Math.max(0, (px - f.left) / (f.right - f.left)));
    const k = Math.round(p * (g.envelope.length - 1));
    const [pp, vv] = g.envelope[k];
    setCursor(pp);
    setTip({
      x: sx(f, pp),
      y: sy(f, vv),
      content: (
        <>
          {g.x_label} = {engine.formatNum(pp)}
          <br />
          Envelope = {engine.formatNum(vv)}
        </>
      ),
    });
  };

  return (
    <div className="graph-view">
      <div className="graph-head">
        <div>
          <p className="graph-summary">{g.heading}</p>
          <p className="ws-line">{g.optimum}</p>
          <p className="hint">{g.explanation}</p>
        </div>
      </div>
      <Legend items={[...g.labels.map((l, i) => ({ label: l, color: series(i) })), { label: 'Envelope', color: 'var(--ink)' }]} />
      <div className="chart" ref={ref}>
        <svg key={key} width={width} height={height} role="img" aria-label={`${g.heading}. ${g.optimum}`}>
          <Axes f={f} xLabel={g.x_label} yLabel="Pagamento esperado" />
          {g.lines.map(([v0, v1], k) => (
            <line
              key={k}
              className={`series-line${seriesDash(k) ? "" : " draw-line"}`}
              pathLength={1}
              x1={sx(f, 0)}
              y1={sy(f, v0)}
              x2={sx(f, 1)}
              y2={sy(f, v1)}
              stroke={series(k)}
              strokeDasharray={seriesDash(k)}
              style={{ animationDelay: `${k * 90}ms` }}
            />
          ))}
          {g.lines.map(([, v1], k) => (
            <text key={`l${k}`} className="direct-label" x={sx(f, 1) - 6} y={sy(f, v1) - 7} textAnchor="end">
              {g.labels[k]}
            </text>
          ))}
          <polyline className="envelope draw-line" pathLength={1} points={env} style={{ animationDelay: `${linesEnd}ms` }} />
          {cursor !== null ? <line className="crosshair" x1={sx(f, cursor)} x2={sx(f, cursor)} y1={f.top} y2={f.bottom} /> : null}
          <g className="optimum" style={{ animationDelay: `${linesEnd + 700}ms` }}>
            <circle className="optimum-dot" cx={ox} cy={oy} r={6} />
            <path className="pen-stroke draw-line" pathLength={1} d={penLoop(key, ox, oy, 17, 15)} style={{ animationDelay: `${linesEnd + 700}ms` }} />
            <text className="optimum-label" x={ox + (ox > f.right - 170 ? -22 : 22)} y={oy - 16} textAnchor={ox > f.right - 170 ? 'end' : 'start'}>
              {g.marker}
            </text>
          </g>
          <rect
            className="hover-layer"
            x={f.left}
            y={f.top}
            width={f.right - f.left}
            height={f.bottom - f.top}
            onPointerMove={onMove}
            onPointerLeave={() => {
              setCursor(null);
              setTip(null);
            }}
          />
        </svg>
        <TipBox tip={tip} width={width} />
      </div>
    </div>
  );
}
