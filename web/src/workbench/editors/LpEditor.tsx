import * as engine from '../../engine';
import type { LpInput, ModuleKey } from '../../types';
import { CellInput, GridTable, setAt, setGrid, Stepper, TitleField } from './common';

interface Props {
  module: ModuleKey;
  input: LpInput;
  onChange: (v: LpInput) => void;
  maxSize: number;
  hint: string;
}

const RELS = ['<=', '>=', '='];

export function LpEditor({ module, input, onChange, maxSize, hint }: Props) {
  const integer = module === 'integer_programming';
  const nv = input.nvars;
  const nc = input.ncons;
  const resize = (part: Partial<LpInput>) => onChange(engine.fit(module, { ...input, ...part }));
  const varName = (j: number) => input.var_names[j] || `x${j + 1}`;
  return (
    <>
      <div className="editor-head">
        <TitleField value={input.title} onChange={(title) => onChange({ ...input, title })} />
        <div className="editor-sizes">
          <Stepper label="Variáveis" value={nv} min={1} max={maxSize} onChange={(v) => resize({ nvars: v })} />
          <Stepper label="Restrições" value={nc} min={1} max={maxSize} onChange={(v) => resize({ ncons: v })} />
        </div>
        <p className="hint">{hint}</p>
      </div>
      <GridTable caption="Modelo de programação linear" wide>
        <thead>
          <tr>
            <th scope="col">
              <span className="sr-only">Linha do modelo</span>
            </th>
            {Array.from({ length: nv }, (_, j) => (
              <th key={j} scope="col">
                <CellInput r={0} c={j + 1} kind="name" value={input.var_names[j] ?? ''} label={`Nome da variável ${j + 1}`} onChange={(v) => onChange({ ...input, var_names: setAt(input.var_names, j, v) })} />
              </th>
            ))}
            <th scope="col" className="col-label">
              Tipo
            </th>
            <th scope="col" className="col-label">
              LD
            </th>
          </tr>
        </thead>
        <tbody>
          <tr className="row-objective">
            <th scope="row">
              <div className="sense-toggle" role="radiogroup" aria-label="Sentido do objetivo">
                {[true, false].map((max) => (
                  <button
                    key={String(max)}
                    type="button"
                    role="radio"
                    aria-checked={input.maximize === max}
                    className={input.maximize === max ? 'is-on' : undefined}
                    onClick={() => onChange({ ...input, maximize: max })}
                  >
                    {max ? 'Maximizar' : 'Minimizar'}
                  </button>
                ))}
              </div>
            </th>
            {Array.from({ length: nv }, (_, j) => (
              <td key={j}>
                <CellInput r={1} c={j + 1} value={input.obj[j] ?? ''} label={`Coeficiente de ${varName(j)} no objetivo`} onChange={(v) => onChange({ ...input, obj: setAt(input.obj, j, v) })} />
              </td>
            ))}
            <td />
            <td />
          </tr>
          {Array.from({ length: nc }, (_, i) => (
            <tr key={i}>
              <th scope="row">
                <CellInput r={i + 2} c={0} kind="name" width="wide" value={input.con_names[i] ?? ''} label={`Nome da restrição ${i + 1}`} onChange={(v) => onChange({ ...input, con_names: setAt(input.con_names, i, v) })} />
              </th>
              {Array.from({ length: nv }, (_, j) => (
                <td key={j}>
                  <CellInput r={i + 2} c={j + 1} value={input.a[i]?.[j] ?? ''} label={`Restrição ${i + 1}, coeficiente de ${varName(j)}`} onChange={(v) => onChange({ ...input, a: setGrid(input.a, i, j, v) })} />
                </td>
              ))}
              <td>
                <select
                  className="cell-select"
                  data-r={i + 2}
                  data-c={nv + 1}
                  value={input.rel[i] ?? '<='}
                  aria-label={`Tipo da restrição ${i + 1}`}
                  onChange={(e) => onChange({ ...input, rel: setAt(input.rel, i, e.target.value) })}
                >
                  {RELS.map((r) => (
                    <option key={r} value={r}>
                      {r === '<=' ? '≤' : r === '>=' ? '≥' : '='}
                    </option>
                  ))}
                </select>
              </td>
              <td className="col-rhs">
                <CellInput r={i + 2} c={nv + 2} value={input.rhs[i] ?? ''} label={`Lado direito da restrição ${i + 1}`} onChange={(v) => onChange({ ...input, rhs: setAt(input.rhs, i, v) })} />
              </td>
            </tr>
          ))}
          <tr className="row-bounds row-bounds--first">
            <th scope="row" className="row-label">
              Limite inferior
            </th>
            {Array.from({ length: nv }, (_, j) => (
              <td key={j}>
                {input.unrestricted[j] ? (
                  <span className="cell-fixed">−∞</span>
                ) : (
                  <CellInput r={nc + 2} c={j + 1} value={input.lower[j] ?? ''} label={`Limite inferior de ${varName(j)}`} onChange={(v) => onChange({ ...input, lower: setAt(input.lower, j, v) })} />
                )}
              </td>
            ))}
            <td colSpan={2} />
          </tr>
          <tr className="row-bounds">
            <th scope="row" className="row-label">
              Limite superior
            </th>
            {Array.from({ length: nv }, (_, j) => (
              <td key={j}>
                <CellInput r={nc + 3} c={j + 1} value={input.upper[j] ?? ''} label={`Limite superior de ${varName(j)}`} onChange={(v) => onChange({ ...input, upper: setAt(input.upper, j, v) })} />
              </td>
            ))}
            <td colSpan={2} />
          </tr>
          <tr className="row-bounds">
            <th scope="row" className="row-label">
              Irrestrita?
            </th>
            {Array.from({ length: nv }, (_, j) => (
              <td key={j} className="cell-check">
                <input
                  type="checkbox"
                  checked={!!input.unrestricted[j]}
                  aria-label={`${varName(j)} irrestrita`}
                  onChange={(e) => onChange({ ...input, unrestricted: setAt(input.unrestricted, j, e.target.checked) })}
                />
              </td>
            ))}
            <td colSpan={2} />
          </tr>
          {integer ? (
            <tr className="row-bounds">
              <th scope="row" className="row-label">
                Inteira?
              </th>
              {Array.from({ length: nv }, (_, j) => (
                <td key={j} className="cell-check">
                  <input
                    type="checkbox"
                    checked={!!input.integer[j]}
                    aria-label={`${varName(j)} inteira`}
                    onChange={(e) => onChange({ ...input, integer: setAt(input.integer, j, e.target.checked) })}
                  />
                </td>
              ))}
              <td colSpan={2} />
            </tr>
          ) : null}
        </tbody>
      </GridTable>
    </>
  );
}
