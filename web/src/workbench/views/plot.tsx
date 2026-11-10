import { useLayoutEffect, useRef, useState, type ReactNode } from 'react';

export function niceStep(range: number): number {
  const raw = range / 8;
  if (!(raw > 0) || !Number.isFinite(raw)) return 1;
  const mag = Math.pow(10, Math.floor(Math.log10(raw)));
  const r = raw / mag;
  const s = r < 1.5 ? 1 : r < 3 ? 2 : r < 7 ? 5 : 10;
  return s * mag;
}

export function ticks(min: number, max: number): number[] {
  const step = niceStep(max - min);
  const out: number[] = [];
  let v = Math.ceil(min / step) * step;
  let guard = 0;
  while (v <= max + 1e-9 && guard < 200) {
    out.push(Math.abs(v) < 1e-12 ? 0 : v);
    v += step;
    guard++;
  }
  return out;
}

export function short(v: number): string {
  if (Math.abs(v - Math.round(v)) < 1e-9) return String(Math.round(v));
  return v.toFixed(2).replace(/0+$/, '').replace(/\.$/, '');
}

export function useWidth<T extends HTMLElement>(fallback = 640) {
  const ref = useRef<T>(null);
  const [width, setWidth] = useState(fallback);
  useLayoutEffect(() => {
    const el = ref.current;
    if (!el) return;
    setWidth(el.clientWidth || fallback);
    const ro = new ResizeObserver((entries) => {
      const w = entries[0]?.contentRect.width;
      if (w) setWidth(w);
    });
    ro.observe(el);
    return () => ro.disconnect();
  }, [fallback]);
  return [ref, width] as const;
}

export interface Frame {
  width: number;
  height: number;
  left: number;
  right: number;
  top: number;
  bottom: number;
  xmin: number;
  xmax: number;
  ymin: number;
  ymax: number;
}

export function makeFrame(width: number, height: number, xmin: number, xmax: number, ymin: number, ymax: number, compact = false): Frame {
  return {
    width,
    height,
    left: compact ? 40 : 56,
    right: width - (compact ? 14 : 22),
    top: compact ? 18 : 24,
    bottom: height - 46,
    xmin,
    xmax,
    ymin,
    ymax,
  };
}

export function sx(f: Frame, x: number): number {
  return f.left + ((x - f.xmin) / (f.xmax - f.xmin || 1)) * (f.right - f.left);
}

export function sy(f: Frame, y: number): number {
  return f.bottom - ((y - f.ymin) / (f.ymax - f.ymin || 1)) * (f.bottom - f.top);
}

export function Axes({ f, xLabel, yLabel }: { f: Frame; xLabel: string; yLabel: string }) {
  const xt = ticks(f.xmin, f.xmax);
  const yt = ticks(f.ymin, f.ymax);
  const x0 = f.xmin <= 0 && f.xmax >= 0 ? 0 : f.xmin;
  const y0 = f.ymin <= 0 && f.ymax >= 0 ? 0 : f.ymin;
  return (
    <g className="axes">
      {xt.map((x) => (
        <g key={`x${x}`}>
          <line className="grid-line" x1={sx(f, x)} x2={sx(f, x)} y1={f.top} y2={f.bottom} />
          <text className="tick" x={sx(f, x)} y={f.bottom + 16} textAnchor="middle">
            {short(x)}
          </text>
        </g>
      ))}
      {yt.map((y) => (
        <g key={`y${y}`}>
          <line className="grid-line" x1={f.left} x2={f.right} y1={sy(f, y)} y2={sy(f, y)} />
          {y === f.ymin && xt.includes(f.xmin) && f.xmin === f.ymin ? null : (
            <text className="tick" x={f.left - 8} y={sy(f, y) + 4} textAnchor="end">
              {short(y)}
            </text>
          )}
        </g>
      ))}
      <line className="axis-line" x1={sx(f, f.xmin)} x2={sx(f, f.xmax)} y1={sy(f, y0)} y2={sy(f, y0)} />
      <line className="axis-line" x1={sx(f, x0)} x2={sx(f, x0)} y1={sy(f, f.ymin)} y2={sy(f, f.ymax)} />
      <text className="axis-label" x={(f.left + f.right) / 2} y={f.height - 6} textAnchor="middle">
        {xLabel}
      </text>
      <text className="axis-label" x={f.left - 44 < 6 ? 6 : 10} y={f.top - 8}>
        {yLabel}
      </text>
    </g>
  );
}

export interface Tip {
  x: number;
  y: number;
  content: ReactNode;
}

export function TipBox({ tip, width }: { tip: Tip | null; width: number }) {
  if (!tip) return null;
  const flip = tip.x > width - 200;
  return (
    <div className="chart-tip" style={{ left: tip.x, top: tip.y, transform: `translate(${flip ? 'calc(-100% - 12px)' : '12px'}, -50%)` }} role="status">
      {tip.content}
    </div>
  );
}

export function Legend({ items }: { items: { label: string; color: string; off?: boolean; onToggle?: () => void }[] }) {
  return (
    <ul className="legend">
      {items.map((it, i) => (
        <li key={i}>
          {it.onToggle ? (
            <label className={`legend-item${it.off ? ' is-off' : ''}`}>
              <input type="checkbox" checked={!it.off} onChange={it.onToggle} />
              <span className="swatch" style={{ background: it.color }} aria-hidden="true" />
              {it.label}
            </label>
          ) : (
            <span className="legend-item">
              <span className="swatch" style={{ background: it.color }} aria-hidden="true" />
              {it.label}
            </span>
          )}
        </li>
      ))}
    </ul>
  );
}

export const SERIES = ['var(--s1)', 'var(--s2)', 'var(--s3)', 'var(--s4)', 'var(--s5)', 'var(--s6)'];

export function series(i: number): string {
  return i < SERIES.length ? SERIES[i] : 'var(--ink-3)';
}

export function seriesDash(i: number): string | undefined {
  return i < SERIES.length ? undefined : '6 4';
}
