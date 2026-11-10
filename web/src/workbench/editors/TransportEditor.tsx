import * as engine from '../../engine';
import type { Choice, Options, TransportInput } from '../../types';
import { CellInput, ChoiceGroup, GridTable, setAt, setGrid, Stepper, TitleField } from './common';

interface Props {
  input: TransportInput;
  options: Options;
  onChange: (v: TransportInput) => void;
  onOptions: (o: Options) => void;
  maxSize: number;
  methods: Choice[];
}

export function TransportEditor({ input, options, onChange, onOptions, maxSize, methods }: Props) {
  const { ns, nd } = input;
  const resize = (part: Partial<TransportInput>) => onChange(engine.fit('transportation', { ...input, ...part }));
  return (
    <>
      <div className="editor-head">
        <TitleField value={input.title} onChange={(title) => onChange({ ...input, title })} />
        <div className="editor-sizes">
          <Stepper label="Origens" value={ns} min={1} max={maxSize} onChange={(v) => resize({ ns: v })} />
          <Stepper label="Destinos" value={nd} min={1} max={maxSize} onChange={(v) => resize({ nd: v })} />
        </div>
        <ChoiceGroup label="Solução inicial" name="start-method" value={options.method ?? 'north_west'} options={methods} onChange={(method) => onOptions({ ...options, method })} />
      </div>
      <GridTable caption="Custos unitários, oferta e demanda" wide>
        <thead>
          <tr>
            <th scope="col" className="col-label">
              Custo unitário
            </th>
            {Array.from({ length: nd }, (_, j) => (
              <th key={j} scope="col">
                <CellInput r={0} c={j + 1} kind="name" value={input.dests[j] ?? ''} label={`Nome do destino ${j + 1}`} onChange={(v) => onChange({ ...input, dests: setAt(input.dests, j, v) })} />
              </th>
            ))}
            <th scope="col" className="col-label">
              Oferta
            </th>
          </tr>
        </thead>
        <tbody>
          {Array.from({ length: ns }, (_, i) => (
            <tr key={i}>
              <th scope="row">
                <CellInput r={i + 1} c={0} kind="name" width="wide" value={input.sources[i] ?? ''} label={`Nome da origem ${i + 1}`} onChange={(v) => onChange({ ...input, sources: setAt(input.sources, i, v) })} />
              </th>
              {Array.from({ length: nd }, (_, j) => (
                <td key={j}>
                  <CellInput r={i + 1} c={j + 1} value={input.cost[i]?.[j] ?? ''} label={`Custo de ${input.sources[i] || `origem ${i + 1}`} para ${input.dests[j] || `destino ${j + 1}`}`} onChange={(v) => onChange({ ...input, cost: setGrid(input.cost, i, j, v) })} />
                </td>
              ))}
              <td className="col-rhs">
                <CellInput r={i + 1} c={nd + 1} value={input.supply[i] ?? ''} label={`Oferta de ${input.sources[i] || `origem ${i + 1}`}`} onChange={(v) => onChange({ ...input, supply: setAt(input.supply, i, v) })} />
              </td>
            </tr>
          ))}
          <tr className="row-bounds row-bounds--first">
            <th scope="row" className="row-label">
              Demanda
            </th>
            {Array.from({ length: nd }, (_, j) => (
              <td key={j}>
                <CellInput r={ns + 1} c={j + 1} value={input.demand[j] ?? ''} label={`Demanda de ${input.dests[j] || `destino ${j + 1}`}`} onChange={(v) => onChange({ ...input, demand: setAt(input.demand, j, v) })} />
              </td>
            ))}
            <td />
          </tr>
        </tbody>
      </GridTable>
    </>
  );
}
