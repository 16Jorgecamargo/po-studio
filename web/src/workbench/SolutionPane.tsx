import { useId } from 'react';
import * as engine from '../engine';
import { useModule, useStudio, type ViewKey } from '../store';
import type { GameInput, LpInput, ModuleKey, NetInput } from '../types';
import { IterationPlayer } from './results/IterationPlayer';
import { SectionList, SectionView } from './results/SectionView';
import { LIVE_GRAPH, VIEWS } from './moduleViews';
import { BbTreeView } from './views/BbTreeView';
import { GameGraphView } from './views/GameGraphView';
import { GanttView } from './views/GanttView';
import { LpGraphView } from './views/LpGraphView';
import { NetworkView } from './views/NetworkView';

const LP_LEGEND = 'Marca-texto verde: variável que entra. Vermelho: variável que sai. Círculo azul: pivô.';

function Empty({ module, view }: { module: ModuleKey; view: ViewKey }) {
  const loadExample = useStudio((s) => s.loadExample);
  const text =
    view === 'graph' && module === 'integer_programming'
      ? 'Clique em Resolver para gerar a árvore do Branch & Bound.'
      : view === 'graph'
        ? 'Clique em Resolver para gerar o gráfico de barras.'
        : view === 'iterations'
          ? module === 'queuing'
            ? 'Clique em Resolver para calcular as probabilidades.'
            : 'Clique em Resolver para gerar as iterações.'
          : 'Preencha os dados e clique em Resolver.';
  return (
    <div className="empty">
      <h3>Ainda sem solução</h3>
      <p>{text}</p>
      <button type="button" className="btn btn--small" onClick={() => loadExample(module)}>
        Carregar o exemplo
      </button>
    </div>
  );
}

function Deadline({ module }: { module: ModuleKey }) {
  const state = useModule(module);
  const patch = useStudio((s) => s.patch);
  const id = useId();
  const stats = state.outcome?.view.pathStats;
  if (!stats) return null;
  const text = state.deadline.trim() ? engine.deadlineProbability(stats.mean, stats.sd, state.deadline) : '';
  return (
    <section className="ws-section deadline">
      <header className="ws-section-head">
        <h3>Probabilidade de concluir o projeto até um prazo</h3>
      </header>
      <div className="deadline-row">
        <div className="field">
          <label className="field-label" htmlFor={id}>
            Prazo T
          </label>
          <input id={id} className="text-input" inputMode="decimal" value={state.deadline} onChange={(e) => patch(module, { deadline: e.target.value })} placeholder="Ex.: 20" />
        </div>
        <p className="deadline-out" aria-live="polite">
          {text}
        </p>
      </div>
    </section>
  );
}

function LpMethod({ module }: { module: ModuleKey }) {
  const state = useModule(module);
  const rebuild = useStudio((s) => s.rebuildLpIterations);
  const cat = engine.catalog();
  const method = state.options.method ?? 'auto';
  return (
    <div className="lp-method">
      <fieldset className="choice-field">
        <legend className="field-label">Método das iterações</legend>
        <div className="choice-row">
          {cat.iterMethods.map((m) => (
            <label key={m.key} className="choice">
              <input type="radio" name="iter-method" checked={method === m.key} onChange={() => rebuild(module, { ...state.options, method: m.key })} />
              {m.label}
            </label>
          ))}
        </div>
      </fieldset>
      {method === 'm' ? (
        <div className="field big-m">
          <label className="field-label" htmlFor="big-m">
            Valor de M
          </label>
          <input id="big-m" className="text-input" inputMode="decimal" value={state.options.big_m ?? '100'} onChange={(e) => rebuild(module, { ...state.options, big_m: e.target.value })} />
        </div>
      ) : null}
    </div>
  );
}

export function SolutionPane({ module }: { module: ModuleKey }) {
  const state = useModule(module);
  const patch = useStudio((s) => s.patch);
  const views = VIEWS[module];
  const view = views.some((v) => v.key === state.view) ? state.view : 'results';
  const out = state.outcome;
  const tabsId = useId();

  const setView = (v: ViewKey) => patch(module, { view: v });
  const onTabKey = (e: React.KeyboardEvent, i: number) => {
    let next = -1;
    if (e.key === 'ArrowRight') next = (i + 1) % views.length;
    if (e.key === 'ArrowLeft') next = (i - 1 + views.length) % views.length;
    if (e.key === 'Home') next = 0;
    if (e.key === 'End') next = views.length - 1;
    if (next >= 0) {
      e.preventDefault();
      setView(views[next].key);
      document.getElementById(`${tabsId}-${views[next].key}`)?.focus();
    }
  };

  let body: React.ReactNode = null;
  if (view === 'graph' && LIVE_GRAPH[module]) {
    if (module === 'linear_programming') {
      body = (
        <LpGraphView
          input={state.input as LpInput}
          hidden={state.hiddenLines}
          onHidden={(h) => patch(module, { hiddenLines: h })}
          showPath={state.showPath}
          onShowPath={(v) => patch(module, { showPath: v })}
        />
      );
    } else if (module === 'zero_sum_game') {
      body = <GameGraphView input={state.input as GameInput} />;
    } else {
      const n = (state.input as NetInput).n;
      body = (
        <NetworkView
          module={module}
          input={state.input as NetInput}
          start={Math.min(state.options.start ?? 0, n - 1)}
          target={Math.min(state.options.target ?? 4, n - 1)}
          view={out && !state.stale ? out.view : null}
        />
      );
    }
  } else if (!out) {
    body = <Empty module={module} view={view} />;
  } else if (view === 'results') {
    body = (
      <>
        <SectionList sections={out.results} id={`${module}-res`} />
        {module === 'pert' ? <Deadline module={module} /> : null}
      </>
    );
  } else if (view === 'iterations') {
    if (module === 'queuing') {
      const names = out.view.scenarios ?? [];
      const i = Math.min(state.queueSelected, out.iterations.length - 1);
      body = (
        <>
          <div className="choice-row scenario-pick" role="radiogroup" aria-label="Cenário">
            <span className="field-label">Cenário</span>
            {out.iterations.map((_, k) => (
              <label key={k} className="choice">
                <input type="radio" name="scenario" checked={i === k} onChange={() => patch(module, { queueSelected: k })} />
                {names[k] || k + 1}
              </label>
            ))}
          </div>
          {out.iterations[i] ? <SectionView section={out.iterations[i]} id={`q-det-${i}`} /> : null}
        </>
      );
    } else {
      body = (
        <>
          {module === 'linear_programming' ? <LpMethod module={module} /> : null}
          {out.iterError ? (
            <div className="error-note" role="alert">
              <strong>Atenção:</strong> {out.iterError}
            </div>
          ) : null}
          {out.iterations.length ? (
            <IterationPlayer
              id={`${module}-it`}
              sections={out.iterations}
              shown={state.shown}
              onShown={(k) => patch(module, { shown: k })}
              legend={module === 'linear_programming' || module === 'integer_programming' || module === 'transportation' ? LP_LEGEND : undefined}
              unit={module === 'integer_programming' ? 'Subproblema' : module === 'cpm' || module === 'pert' ? 'Cálculo' : 'Quadro'}
            />
          ) : out.iterError ? null : (
            <p className="notice">Este problema não gerou iterações.</p>
          )}
        </>
      );
    }
  } else if (view === 'graph') {
    if (module === 'integer_programming' && out.view.tree) {
      body = <BbTreeView nodes={out.view.tree.nodes} best={out.view.tree.best} selected={state.bbSelected} onSelect={(id) => patch(module, { bbSelected: id })} />;
    } else if ((module === 'cpm' || module === 'pert') && out.view.activities) {
      body = (
        <GanttView
          activities={out.view.activities}
          duration={out.view.duration ?? 0}
          delayAct={state.delayAct}
          delay={state.delay}
          onDelay={(act, delay) => patch(module, { delayAct: act, delay })}
        />
      );
    }
  }

  return (
    <section className="sheet sheet--solution" aria-labelledby="solution-title">
      <header className="sheet-head sheet-head--solution">
        <h2 id="solution-title">Solução</h2>
        <div className="view-tabs" role="tablist" aria-label="Partes da solução">
          {views.map((v, i) => (
            <button
              key={v.key}
              id={`${tabsId}-${v.key}`}
              type="button"
              role="tab"
              aria-selected={view === v.key}
              aria-controls={`${tabsId}-panel`}
              tabIndex={view === v.key ? 0 : -1}
              className={`view-tab${view === v.key ? ' is-on' : ''}`}
              onClick={() => setView(v.key)}
              onKeyDown={(e) => onTabKey(e, i)}
            >
              {v.label}
            </button>
          ))}
        </div>
      </header>
      <div id={`${tabsId}-panel`} className="sheet-body solution-body" role="tabpanel" aria-labelledby={`${tabsId}-${view}`} key={`${view}-${state.solvedAt}`}>
        {state.stale && out ? (
          <p className="stale-note">O enunciado mudou depois da última resolução. Clique em Resolver para atualizar.</p>
        ) : null}
        {state.notice && out ? <p className="notice">{state.notice}</p> : null}
        {body}
      </div>
    </section>
  );
}
