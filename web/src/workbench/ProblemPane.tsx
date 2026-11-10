import * as engine from '../engine';
import { useModule, useStudio } from '../store';
import type { AlgebraInput, AnyInput, GameInput, LpInput, ModuleKey, NetInput, Options, RowsInput, TransportInput } from '../types';
import { mod } from '../ui/platform';
import { AlgebraEditor } from './editors/AlgebraEditor';
import { GameEditor } from './editors/GameEditor';
import { LpEditor } from './editors/LpEditor';
import { NetworkEditor } from './editors/NetworkEditor';
import { RowsEditor } from './editors/RowsEditor';
import { TransportEditor } from './editors/TransportEditor';

interface Props {
  module: ModuleKey;
  onSolved: () => void;
}

export function ProblemPane({ module, onSolved }: Props) {
  const state = useModule(module);
  const setInput = useStudio((s) => s.setInput);
  const setOptions = useStudio((s) => s.setOptions);
  const solve = useStudio((s) => s.solve);
  const cat = engine.catalog();
  const info = cat.modules.find((m) => m.key === module)!;
  const onChange = (v: AnyInput) => setInput(module, v);
  const onOptions = (o: Options) => setOptions(module, o);
  const max = cat.maxSize;

  const run = () => {
    if (solve(module)) onSolved();
  };

  let editor: React.ReactNode;
  switch (module) {
    case 'linear_equations':
    case 'matrix_inverse':
      editor = <AlgebraEditor module={module} input={state.input as AlgebraInput} onChange={onChange} maxSize={max} />;
      break;
    case 'linear_programming':
    case 'integer_programming':
      editor = <LpEditor module={module} input={state.input as LpInput} onChange={onChange} maxSize={max} hint={info.hint} />;
      break;
    case 'transportation':
      editor = <TransportEditor input={state.input as TransportInput} options={state.options} onChange={onChange} onOptions={onOptions} maxSize={max} methods={cat.startMethods} />;
      break;
    case 'minimal_spanning_tree':
    case 'shortest_route':
    case 'maximal_flow':
      editor = <NetworkEditor module={module} input={state.input as NetInput} options={state.options} onChange={onChange} onOptions={onOptions} maxSize={max} hint={info.hint} />;
      break;
    case 'cpm':
    case 'pert':
      editor = (
        <RowsEditor
          module={module}
          input={state.input as RowsInput}
          onChange={onChange}
          headers={module === 'pert' ? cat.pertHeaders : cat.cpmHeaders}
          countLabel="Nº de atividades"
          hint={info.hint}
          maxSize={max}
        />
      );
      break;
    case 'queuing':
      editor = <RowsEditor module={module} input={state.input as RowsInput} onChange={onChange} headers={cat.queueHeaders} countLabel="Nº de cenários" hint={info.hint} maxSize={max} />;
      break;
    case 'zero_sum_game':
      editor = <GameEditor input={state.input as GameInput} onChange={onChange} maxSize={max} hint={info.hint} />;
      break;
  }

  return (
    <section
      className="sheet sheet--problem"
      aria-labelledby="problem-title"
      onKeyDown={(e) => {
        if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
          e.preventDefault();
          run();
        }
      }}
    >
      <header className="sheet-head">
        <p className="sheet-kicker">Enunciado</p>
        <h1 id="problem-title">{info.name}</h1>
        <p className="sheet-blurb">{info.blurb}</p>
      </header>
      <div className="sheet-body">{editor}</div>
      <footer className="solve-bar">
        {state.error ? (
          <div className="error-note" role="alert">
            <strong>Atenção:</strong> {state.error}
          </div>
        ) : null}
        <div className="solve-row">
          <button type="button" className="btn btn--primary btn--solve" onClick={run}>
            Resolver
          </button>
          <span className="hint solve-kbd">
            ou <kbd>{mod('Enter')}</kbd> dentro do enunciado
          </span>
        </div>
      </footer>
    </section>
  );
}
