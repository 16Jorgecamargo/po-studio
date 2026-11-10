import { useId, useState } from 'react';
import * as engine from '../../engine';
import type { ActivitySchedule } from '../../types';
import { TipBox, useWidth, type Tip } from './plot';

interface Props {
  activities: ActivitySchedule[];
  duration: number;
  delayAct: number;
  delay: number;
  onDelay: (act: number, delay: number) => void;
}

export function GanttView({ activities, duration, delayAct, delay, onDelay }: Props) {
  const [ref, width] = useWidth<HTMLDivElement>();
  const [tip, setTip] = useState<Tip | null>(null);
  const selId = useId();
  const rangeId = useId();
  if (activities.length === 0) return <p className="notice">Nenhuma atividade para desenhar.</p>;

  const act = Math.min(delayAct, activities.length - 1);
  const a = activities[act];
  const message = engine.delayMessage(activities, act, delay);
  const total = Math.max(duration, 1);
  const rowH = 32;
  const labelW = Math.min(130, Math.max(70, width * 0.18));
  const left = labelW;
  const right = width - 18;
  const top = 32;
  const height = activities.length * rowH + top + 14;
  const x = (t: number) => left + (t / total) * (right - left);
  const steps = Array.from({ length: 11 }, (_, k) => (total * k) / 10);

  return (
    <div className="graph-view">
      <div className="delay-panel">
        <div className="field">
          <label className="field-label" htmlFor={selId}>
            Experimente mudanças: alterar início de
          </label>
          <select id={selId} className="text-input" value={act} onChange={(e) => onDelay(Number(e.target.value), 0)}>
            {activities.map((it, i) => (
              <option key={i} value={i}>
                {it.name}
                {it.critical ? ' (crítica)' : ''}
              </option>
            ))}
          </select>
        </div>
        {a.critical ? (
          <p className="pen-red">Atividade crítica: seu cronograma não pode ser alterado.</p>
        ) : (
          <div className="field delay-range">
            <label className="field-label" htmlFor={rangeId}>
              Atraso: <span className="num-text">{engine.formatNum(delay)}</span> <span className="hint">(máximo = folga total = {engine.formatNum(a.total_float)})</span>
            </label>
            <input id={rangeId} type="range" min={0} max={a.total_float} step="any" value={delay} onChange={(e) => onDelay(act, Number(e.target.value))} />
          </div>
        )}
        {message ? <p className="pen-red delay-message">{message}</p> : null}
      </div>
      <div className="chart" ref={ref}>
        <svg width={width} height={height} role="img" aria-label={`Gráfico de barras do projeto, duração ${engine.formatNum(duration)}`}>
          {steps.map((t, k) => (
            <g key={k}>
              <line className="grid-line" x1={x(t)} x2={x(t)} y1={top - 6} y2={height - 8} />
              <text className="tick" x={x(t)} y={16} textAnchor="middle">
                {engine.formatNum(t)}
              </text>
            </g>
          ))}
          {activities.map((it, i) => {
            const y = top + i * rowH;
            const shift = i === act ? delay : 0;
            const bx = x(it.es + shift);
            const bw = Math.max(2, x(it.ef + shift) - bx);
            const show = () =>
              setTip({
                x: bx + bw / 2,
                y: y + rowH / 2,
                content: (
                  <>
                    <strong>{it.name}</strong>
                    {it.critical ? ' (crítica)' : ''}
                    <br />
                    Início cedo {engine.formatNum(it.es)}, término cedo {engine.formatNum(it.ef)}
                    <br />
                    Início tarde {engine.formatNum(it.ls)}, término tarde {engine.formatNum(it.lf)}
                    <br />
                    Folga total {engine.formatNum(it.total_float)}, folga livre {engine.formatNum(it.free_float)}
                  </>
                ),
              });
            return (
              <g key={i}>
                <text className={`gantt-name${it.critical ? ' is-critical' : ''}`} x={10} y={y + rowH / 2 + 5}>
                  {it.name}
                </text>
                {!it.critical ? <rect className="float-band" x={x(it.es)} y={y + 9} width={Math.max(0, x(it.lf) - x(it.es))} height={rowH - 18} rx={2} /> : null}
                <rect
                  className={`gantt-bar grow-x${it.critical ? ' is-critical' : ''}`}
                  x={bx}
                  y={y + 5}
                  width={bw}
                  height={rowH - 10}
                  rx={3}
                  style={{ animationDelay: `${Math.min(i * 40, 500)}ms`, transformOrigin: `${bx}px 0` }}
                  tabIndex={0}
                  aria-label={`${it.name}: início ${engine.formatNum(it.es + shift)}, término ${engine.formatNum(it.ef + shift)}${it.critical ? ', crítica' : ''}`}
                  onPointerEnter={show}
                  onPointerLeave={() => setTip(null)}
                  onFocus={show}
                  onBlur={() => setTip(null)}
                />
              </g>
            );
          })}
        </svg>
        <TipBox tip={tip} width={width} />
      </div>
      <ul className="legend">
        <li className="legend-item">
          <span className="swatch" style={{ background: 'var(--red)' }} aria-hidden="true" />
          Atividade crítica
        </li>
        <li className="legend-item">
          <span className="swatch" style={{ background: 'var(--blue)' }} aria-hidden="true" />
          Não crítica
        </li>
        <li className="legend-item">
          <span className="swatch swatch--band" aria-hidden="true" />
          Intervalo em que pode ser realizada (folga total)
        </li>
      </ul>
    </div>
  );
}
