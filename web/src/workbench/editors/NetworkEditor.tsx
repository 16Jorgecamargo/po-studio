import * as engine from '../../engine';
import type { ModuleKey, NetInput, Options } from '../../types';
import { CellInput, ChoiceGroup, GridTable, NodeSelect, setAt, setGrid, Stepper, TitleField } from './common';

interface Props {
  module: ModuleKey;
  input: NetInput;
  options: Options;
  onChange: (v: NetInput) => void;
  onOptions: (o: Options) => void;
  maxSize: number;
  hint: string;
}

export function NetworkEditor({ module, input, options, onChange, onOptions, maxSize, hint }: Props) {
  const n = input.n;
  const names = input.names.map((nm, i) => nm || `Nó ${i + 1}`);
  const start = Math.min(options.start ?? 0, n - 1);
  const target = Math.min(options.target ?? 4, n - 1);
  const setNode = (part: Options) => onOptions({ ...options, start, target, ...part });
  return (
    <>
      <div className="editor-head">
        <TitleField value={input.title} onChange={(title) => onChange({ ...input, title })} />
        <div className="editor-sizes">
          <Stepper label="Nº de nós" value={n} min={2} max={maxSize} onChange={(v) => onChange(engine.fit(module, { ...input, n: v }))} />
          {module === 'minimal_spanning_tree' ? <NodeSelect label="Nó inicial" value={start} names={names} onChange={(v) => setNode({ start: v })} /> : null}
          {module === 'shortest_route' ? (
            <>
              <NodeSelect label="De" value={start} names={names} onChange={(v) => setNode({ start: v })} />
              <NodeSelect label="Para" value={target} names={names} onChange={(v) => setNode({ target: v })} />
            </>
          ) : null}
          {module === 'maximal_flow' ? (
            <>
              <NodeSelect label="Origem" value={start} names={names} onChange={(v) => setNode({ start: v })} />
              <NodeSelect label="Destino" value={target} names={names} onChange={(v) => setNode({ target: v })} />
            </>
          ) : null}
        </div>
        {module === 'shortest_route' ? (
          <ChoiceGroup
            label="Algoritmo"
            name="short-algo"
            value={options.algo ?? 'dijkstra'}
            options={[
              { key: 'dijkstra', label: 'Dijkstra' },
              { key: 'floyd', label: 'Floyd (todas as rotas)' },
            ]}
            onChange={(algo) => setNode({ algo })}
          />
        ) : null}
        <p className="hint">{hint}</p>
      </div>
      <GridTable caption={module === 'maximal_flow' ? 'Capacidades dos arcos' : 'Distâncias entre os nós'} wide>
        <thead>
          <tr>
            <th scope="col" className="col-label">
              De \ Para
            </th>
            {Array.from({ length: n }, (_, j) => (
              <th key={j} scope="col" className="col-name">
                {names[j]}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {Array.from({ length: n }, (_, i) => (
            <tr key={i}>
              <th scope="row">
                <CellInput r={i} c={0} kind="name" value={input.names[i] ?? ''} label={`Nome do nó ${i + 1}`} onChange={(v) => onChange({ ...input, names: setAt(input.names, i, v) })} />
              </th>
              {Array.from({ length: n }, (_, j) => (
                <td key={j}>
                  {i === j ? (
                    <span className="cell-fixed" aria-label="Sem arco">
                      –
                    </span>
                  ) : (
                    <CellInput r={i} c={j + 1} value={input.matrix[i]?.[j] ?? ''} label={`De ${names[i]} para ${names[j]}`} onChange={(v) => onChange({ ...input, matrix: setGrid(input.matrix, i, j, v) })} />
                  )}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </GridTable>
    </>
  );
}
