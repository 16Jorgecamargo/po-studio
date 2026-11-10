import * as engine from '../../engine';
import type { GameInput } from '../../types';
import { CellInput, GridTable, setGrid, Stepper, TitleField } from './common';

interface Props {
  input: GameInput;
  onChange: (v: GameInput) => void;
  maxSize: number;
  hint: string;
}

export function GameEditor({ input, onChange, maxSize, hint }: Props) {
  const { m, n } = input;
  const resize = (part: Partial<GameInput>) => onChange(engine.fit('zero_sum_game', { ...input, ...part }));
  return (
    <>
      <div className="editor-head">
        <TitleField value={input.title} onChange={(title) => onChange({ ...input, title })} />
        <div className="editor-sizes">
          <Stepper label="Estratégias de A" value={m} min={1} max={maxSize} onChange={(v) => resize({ m: v })} />
          <Stepper label="Estratégias de B" value={n} min={1} max={maxSize} onChange={(v) => resize({ n: v })} />
        </div>
        <p className="hint">{hint}</p>
      </div>
      <GridTable caption="Matriz de pagamentos ao jogador A">
        <thead>
          <tr>
            <th scope="col" className="col-label">
              A \ B
            </th>
            {Array.from({ length: n }, (_, j) => (
              <th key={j} scope="col">
                B{j + 1}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {Array.from({ length: m }, (_, i) => (
            <tr key={i}>
              <th scope="row">A{i + 1}</th>
              {Array.from({ length: n }, (_, j) => (
                <td key={j}>
                  <CellInput r={i} c={j} value={input.payoff[i]?.[j] ?? ''} label={`A${i + 1} contra B${j + 1}`} onChange={(v) => onChange({ ...input, payoff: setGrid(input.payoff, i, j, v) })} />
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </GridTable>
    </>
  );
}
