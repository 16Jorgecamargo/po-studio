import { useEffect, useId, useState, type ReactNode } from 'react';

export function setAt<T>(list: T[], i: number, v: T): T[] {
  const next = list.slice();
  next[i] = v;
  return next;
}

export function setGrid<T>(grid: T[][], i: number, j: number, v: T): T[][] {
  const next = grid.slice();
  next[i] = setAt(grid[i], j, v);
  return next;
}

export function TitleField({ value, onChange }: { value: string; onChange: (v: string) => void }) {
  const id = useId();
  return (
    <div className="field title-field">
      <label className="field-label" htmlFor={id}>
        Título do problema
      </label>
      <input id={id} className="text-input" value={value} placeholder="Ex.: Mix de produção" onChange={(e) => onChange(e.target.value)} />
    </div>
  );
}

interface StepperProps {
  label: string;
  value: number;
  min: number;
  max: number;
  onChange: (v: number) => void;
}

export function Stepper({ label, value, min, max, onChange }: StepperProps) {
  const id = useId();
  const [text, setText] = useState(String(value));
  useEffect(() => setText(String(value)), [value]);
  const commit = (raw: string) => {
    const v = parseInt(raw.trim(), 10);
    if (Number.isFinite(v)) {
      const c = Math.min(max, Math.max(min, v));
      if (c !== value) onChange(c);
      setText(String(c));
    } else {
      setText(String(value));
    }
  };
  return (
    <div className="stepper">
      <label className="field-label" htmlFor={id}>
        {label}
      </label>
      <div className="stepper-box">
        <button type="button" onClick={() => onChange(Math.max(min, value - 1))} disabled={value <= min} aria-label={`Diminuir ${label.toLowerCase()}`}>
          −
        </button>
        <input
          id={id}
          inputMode="numeric"
          value={text}
          onChange={(e) => setText(e.target.value)}
          onBlur={(e) => commit(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter') commit((e.target as HTMLInputElement).value);
            if (e.key === 'ArrowUp') {
              e.preventDefault();
              onChange(Math.min(max, value + 1));
            }
            if (e.key === 'ArrowDown') {
              e.preventDefault();
              onChange(Math.max(min, value - 1));
            }
          }}
        />
        <button type="button" onClick={() => onChange(Math.min(max, value + 1))} disabled={value >= max} aria-label={`Aumentar ${label.toLowerCase()}`}>
          +
        </button>
      </div>
    </div>
  );
}

export function ChoiceGroup<T extends string>({
  label,
  name,
  value,
  options,
  onChange,
}: {
  label: string;
  name: string;
  value: T;
  options: { key: T; label: string }[];
  onChange: (v: T) => void;
}) {
  return (
    <fieldset className="choice-field">
      <legend className="field-label">{label}</legend>
      <div className="choice-row">
        {options.map((o) => (
          <label key={o.key} className="choice">
            <input type="radio" name={name} value={o.key} checked={value === o.key} onChange={() => onChange(o.key)} />
            {o.label}
          </label>
        ))}
      </div>
    </fieldset>
  );
}

export function NodeSelect({ label, value, names, onChange }: { label: string; value: number; names: string[]; onChange: (v: number) => void }) {
  const id = useId();
  return (
    <div className="field node-select">
      <label className="field-label" htmlFor={id}>
        {label}
      </label>
      <select id={id} className="text-input" value={value} onChange={(e) => onChange(Number(e.target.value))}>
        {names.map((n, i) => (
          <option key={i} value={i}>
            {n || `Nó ${i + 1}`}
          </option>
        ))}
      </select>
    </div>
  );
}

function moveFocus(e: React.KeyboardEvent<HTMLElement>) {
  const target = e.target as HTMLElement;
  if (!(target instanceof HTMLInputElement) && !(target instanceof HTMLSelectElement)) return;
  const r = Number(target.dataset.r);
  const c = Number(target.dataset.c);
  if (!Number.isFinite(r) || !Number.isFinite(c)) return;
  let dr = 0;
  let dc = 0;
  if (e.key === 'ArrowDown' || (e.key === 'Enter' && !e.shiftKey && !e.metaKey && !e.ctrlKey)) dr = 1;
  else if (e.key === 'ArrowUp' || (e.key === 'Enter' && e.shiftKey)) dr = -1;
  else if (target instanceof HTMLInputElement && e.key === 'ArrowRight' && target.selectionStart === target.value.length) dc = 1;
  else if (target instanceof HTMLInputElement && e.key === 'ArrowLeft' && target.selectionEnd === 0) dc = -1;
  else return;
  if (target instanceof HTMLSelectElement && (e.key === 'ArrowDown' || e.key === 'ArrowUp')) return;
  const grid = e.currentTarget;
  const next = grid.querySelector<HTMLElement>(`[data-r="${r + dr}"][data-c="${c + dc}"]`);
  if (next) {
    e.preventDefault();
    next.focus();
    if (next instanceof HTMLInputElement) next.select();
  }
}

export function GridTable({ caption, children, wide }: { caption: string; children: ReactNode; wide?: boolean }) {
  return (
    <div className={`grid-wrap${wide ? ' grid-wrap--wide' : ''}`} onKeyDown={moveFocus}>
      <table className="grid-edit">
        <caption className="sr-only">{caption}</caption>
        {children}
      </table>
    </div>
  );
}

interface CellProps {
  r: number;
  c: number;
  value: string;
  label: string;
  onChange: (v: string) => void;
  kind?: 'num' | 'name';
  width?: 'narrow' | 'wide';
}

export function CellInput({ r, c, value, label, onChange, kind = 'num', width }: CellProps) {
  return (
    <input
      className={`cell-input cell-input--${kind}${width ? ` cell-input--${width}` : ''}`}
      data-r={r}
      data-c={c}
      value={value}
      aria-label={label}
      inputMode={kind === 'num' ? 'decimal' : undefined}
      autoComplete="off"
      spellCheck={false}
      onChange={(e) => onChange(e.target.value)}
      onFocus={(e) => e.target.select()}
    />
  );
}
