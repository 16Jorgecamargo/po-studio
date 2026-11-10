import * as engine from '../../engine';
import type { AlgebraInput, ModuleKey } from '../../types';
import { CellInput, GridTable, setAt, setGrid, Stepper, TitleField } from './common';

interface Props {
  module: ModuleKey;
  input: AlgebraInput;
  onChange: (v: AlgebraInput) => void;
  maxSize: number;
}

export function AlgebraEditor({ module, input, onChange, maxSize }: Props) {
  const withRhs = module === 'linear_equations';
  const n = input.n;
  return (
    <>
      <div className="editor-head">
        <TitleField value={input.title} onChange={(title) => onChange({ ...input, title })} />
        <div className="editor-sizes">
          <Stepper
            label={withRhs ? 'Nº de equações' : 'Ordem da matriz'}
            value={n}
            min={1}
            max={maxSize}
            onChange={(v) => onChange(engine.fit(module, { ...input, n: v }))}
          />
        </div>
      </div>
      <GridTable caption={withRhs ? 'Coeficientes e lado direito' : 'Elementos da matriz'}>
        <thead>
          <tr>
            <th scope="col">
              <span className="sr-only">Linha</span>
            </th>
            {Array.from({ length: n }, (_, j) => (
              <th key={j} scope="col">
                x{j + 1}
              </th>
            ))}
            {withRhs ? <th scope="col">LD</th> : null}
          </tr>
        </thead>
        <tbody>
          {Array.from({ length: n }, (_, i) => (
            <tr key={i}>
              <th scope="row">{i + 1}</th>
              {Array.from({ length: n }, (_, j) => (
                <td key={j}>
                  <CellInput r={i} c={j} value={input.a[i]?.[j] ?? ''} label={`Linha ${i + 1}, x${j + 1}`} onChange={(v) => onChange({ ...input, a: setGrid(input.a, i, j, v) })} />
                </td>
              ))}
              {withRhs ? (
                <td className="col-rhs">
                  <CellInput r={i} c={n} value={input.b[i] ?? ''} label={`Linha ${i + 1}, lado direito`} onChange={(v) => onChange({ ...input, b: setAt(input.b, i, v) })} />
                </td>
              ) : null}
            </tr>
          ))}
        </tbody>
      </GridTable>
    </>
  );
}
