import { useEffect, useId, useState } from 'react';
import { useStudio } from '../store';
import { ChevronIcon } from '../ui/icons';
import { usePopover } from '../ui/usePopover';

export function FormatMenu() {
  const { open, setOpen, wrapRef, triggerRef } = usePopover();
  const format = useStudio((s) => s.format);
  const setFormat = useStudio((s) => s.setFormat);
  const [decimals, setDecimals] = useState(String(format.decimals));
  const [width, setWidth] = useState(String(format.width));
  const dId = useId();
  const wId = useId();

  useEffect(() => {
    setDecimals(String(format.decimals));
    setWidth(String(format.width));
  }, [format]);

  useEffect(() => {
    if (open) window.setTimeout(() => wrapRef.current?.querySelector<HTMLInputElement>('input[type=radio]:checked')?.focus(), 0);
  }, [open, wrapRef]);

  const commit = (part: Partial<typeof format>) => setFormat({ ...format, ...part });
  const clampInt = (raw: string, min: number, max: number, fallback: number) => {
    const v = parseInt(raw, 10);
    return Number.isFinite(v) ? Math.min(max, Math.max(min, v)) : fallback;
  };

  return (
    <div className="menu-wrap" ref={wrapRef}>
      <button ref={triggerRef} type="button" className="btn btn--quiet bar-btn" aria-haspopup="dialog" aria-expanded={open} onClick={() => setOpen(!open)}>
        Formato numérico
        <ChevronIcon />
      </button>
      {open ? (
        <div className="popover format-pop" role="dialog" aria-label="Formato numérico">
          <fieldset className="choice-field">
            <legend className="field-label">Formato de saída</legend>
            <label className="radio-line">
              <input type="radio" name="numfmt" checked={!format.scientific} onChange={() => commit({ scientific: false })} />
              Decimal (NNNNN.DD)
            </label>
            <label className="radio-line">
              <input type="radio" name="numfmt" checked={format.scientific} onChange={() => commit({ scientific: true })} />
              Notação científica (N.NNNNeDD)
            </label>
          </fieldset>
          <div className="format-nums">
            <div className="field">
              <label className="field-label" htmlFor={dId}>
                Casas decimais (D)
              </label>
              <input
                id={dId}
                className="text-input"
                type="number"
                min={0}
                max={10}
                value={decimals}
                onChange={(e) => setDecimals(e.target.value)}
                onBlur={() => commit({ decimals: clampInt(decimals, 0, 10, format.decimals) })}
                onKeyDown={(e) => e.key === 'Enter' && commit({ decimals: clampInt(decimals, 0, 10, format.decimals) })}
              />
            </div>
            <div className="field">
              <label className="field-label" htmlFor={wId}>
                Dígitos (N)
              </label>
              <input
                id={wId}
                className="text-input"
                type="number"
                min={1}
                max={15}
                value={width}
                onChange={(e) => setWidth(e.target.value)}
                onBlur={() => commit({ width: clampInt(width, 1, 15, format.width) })}
                onKeyDown={(e) => e.key === 'Enter' && commit({ width: clampInt(width, 1, 15, format.width) })}
              />
            </div>
          </div>
          <p className="hint">Os resultados já calculados são refeitos no novo formato.</p>
        </div>
      ) : null}
    </div>
  );
}
