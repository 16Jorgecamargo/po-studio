import { useEffect, useMemo, useState } from 'react';
import * as engine from '../engine';
import type { LpInput } from '../types';
import { penLoop } from '../ui/pen';
import { useReducedMotion } from '../ui/useDisplay';
import { Axes, makeFrame, series, sx, sy, useWidth } from '../workbench/views/plot';

const STEP_MS = 1100;

export function HeroGraph() {
  const input = useMemo(() => engine.example('linear_programming').input as LpInput, []);
  const graph = useMemo(() => engine.lpGraph(input), [input]);
  const path = useMemo(() => engine.lpPath(input), [input]);
  const reduced = useReducedMotion();
  const [ref, width] = useWidth<HTMLDivElement>(480);
  const points = path.ok ? path.value.points : [];
  const [step, setStep] = useState(0);
  const [run, setRun] = useState(0);

  useEffect(() => {
    if (reduced) {
      setStep(points.length - 1);
      return;
    }
    setStep(0);
    let k = 0;
    const timer = window.setInterval(() => {
      k += 1;
      if (k >= points.length) {
        window.clearInterval(timer);
        return;
      }
      setStep(k);
    }, STEP_MS);
    return () => window.clearInterval(timer);
  }, [reduced, points.length, run]);

  if (!graph.ok || points.length === 0) return null;
  const g = graph.value;
  const height = Math.round(Math.min(420, Math.max(280, width * 0.78)));
  const f = makeFrame(width, height, 0, g.xmax, 0, g.ymax, true);
  const cur = points[step];
  const done = step === points.length - 1;
  const fmt = engine.formatNum;
  const model = `Maximizar z = ${input.obj[0]}${input.var_names[0]} + ${input.obj[1]}${input.var_names[1]}`;

  return (
    <figure className="hero-sheet">
      <div className="hero-sheet-head">
        <p className="hero-sheet-title">{input.title}</p>
        <p className="hero-sheet-model">{model}</p>
      </div>
      <div className="chart" ref={ref}>
        <svg width={width} height={height} role="img" aria-label={`Simplex percorrendo os vértices da região viável até o ótimo ${g.marker ?? ''}`}>
          <Axes f={f} xLabel={g.xLabel} yLabel={g.yLabel} />
          {g.region.length >= 3 ? <polygon className="region" points={g.region.map(([x, y]) => `${sx(f, x)},${sy(f, y)}`).join(' ')} /> : null}
          {g.lines.map((l, i) =>
            l.segment ? (
              <g key={i}>
                <line className="series-line" x1={sx(f, l.segment[0][0])} y1={sy(f, l.segment[0][1])} x2={sx(f, l.segment[1][0])} y2={sy(f, l.segment[1][1])} stroke={series(i)} />
                <text className="direct-label" x={sx(f, l.segment[0][0]) + 5} y={sy(f, l.segment[0][1]) - 6}>
                  {l.tag}
                </text>
              </g>
            ) : null,
          )}
          <polyline className="hero-trail" points={points.slice(0, step + 1).map((p) => `${sx(f, p.x[0])},${sy(f, p.x[1])}`).join(' ')} />
          {points.slice(0, step + 1).map((p, i) => (
            <circle key={i} className={`hero-dot${i === step ? ' is-current' : ''}`} cx={sx(f, p.x[0])} cy={sy(f, p.x[1])} r={i === step ? 6.5 : 4} />
          ))}
          {(() => {
            const cxr = g.region.reduce((a, q) => a + q[0], 0) / Math.max(g.region.length, 1);
            const cyr = g.region.reduce((a, q) => a + q[1], 0) / Math.max(g.region.length, 1);
            const px = sx(f, cur.x[0]);
            const py = sy(f, cur.x[1]);
            const dx = px - sx(f, cxr);
            const dy = py - sy(f, cyr);
            const len = Math.hypot(dx, dy) || 1;
            const reach = done ? 30 : 16;
            const lx = px + (dx / len) * reach + (dx >= 0 ? 4 : -4);
            const ly = Math.min(f.bottom - 8, Math.max(f.top + 12, py + (dy / len) * reach + 4));
            return (
              <text key={`z${step}`} className="hero-z" x={lx} y={ly} textAnchor={dx >= 0 ? 'start' : 'end'}>
                z = {fmt(cur.z)}
              </text>
            );
          })()}
          {done ? <path key={`loop${run}`} className="pen-stroke draw-line" pathLength={1} d={penLoop('hero', sx(f, cur.x[0]), sy(f, cur.x[1]), 19, 17)} /> : null}
        </svg>
      </div>
      <figcaption className="hero-caption">
        <span aria-live="polite">
          {step === 0 ? 'O simplex começa na origem.' : done ? `Ótimo em (${fmt(cur.x[0])}, ${fmt(cur.x[1])}), z = ${fmt(cur.z)}.` : `Iteração ${step}: anda para (${fmt(cur.x[0])}, ${fmt(cur.x[1])}).`}
        </span>
        {done && !reduced ? (
          <button type="button" className="btn btn--quiet btn--small" onClick={() => setRun((r) => r + 1)}>
            Ver de novo
          </button>
        ) : null}
      </figcaption>
    </figure>
  );
}
