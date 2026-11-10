import * as engine from '../../engine';
import type { ModuleKey, RowsInput } from '../../types';
import { CellInput, GridTable, setGrid, Stepper, TitleField } from './common';

interface Props {
  module: ModuleKey;
  input: RowsInput;
  onChange: (v: RowsInput) => void;
  headers: string[];
  countLabel: string;
  hint: string;
  maxSize: number;
}

export function RowsEditor({ module, input, onChange, headers, countLabel, hint, maxSize }: Props) {
  return (
    <>
      <div className="editor-head">
        <TitleField value={input.title} onChange={(title) => onChange({ ...input, title })} />
        <div className="editor-sizes">
          <Stepper label={countLabel} value={input.rows.length} min={1} max={maxSize} onChange={(v) => onChange(engine.setCount(module, input, v))} />
        </div>
        <p className="hint">{hint}</p>
      </div>
      <GridTable caption={countLabel.replace('Nº de ', '')} wide>
        <thead>
          <tr>
            {headers.map((h, j) => (
              <th key={j} scope="col" className="col-label">
                {h}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {input.rows.map((row, i) => (
            <tr key={i}>
              {headers.map((h, j) => (
                <td key={j}>
                  <CellInput
                    r={i}
                    c={j}
                    kind={j === 0 ? 'name' : 'num'}
                    width={j === 0 ? 'wide' : undefined}
                    value={row[j] ?? ''}
                    label={`Linha ${i + 1}, ${h}`}
                    onChange={(v) => onChange({ ...input, rows: setGrid(input.rows, i, j, v) })}
                  />
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </GridTable>
    </>
  );
}
