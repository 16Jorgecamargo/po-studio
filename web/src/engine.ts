import init, * as wasm from './wasm/po_wasm.js';
import wasmUrl from './wasm/po_wasm_bg.wasm?url';
import type {
  AnyInput,
  Catalog,
  GameGraph,
  LpGraph,
  LpInput,
  LpPath,
  ModuleKey,
  NetInput,
  NetworkArcs,
  Options,
  Outcome,
  Result,
  ActivitySchedule,
  GameInput,
} from './types';

let ready: Promise<void> | null = null;
let catalogCache: Catalog | null = null;

export function loadEngine(): Promise<void> {
  if (!ready) {
    ready = init({ module_or_path: wasmUrl }).then(() => undefined);
  }
  return ready;
}

function call<T>(text: string): Result<T> {
  try {
    return JSON.parse(text) as Result<T>;
  } catch {
    return { ok: false, error: 'Resposta inválida do motor de cálculo.' };
  }
}

function unwrap<T>(r: Result<T>): T {
  if (!r.ok) throw new Error(r.error);
  return r.value;
}

export function catalog(): Catalog {
  if (!catalogCache) catalogCache = JSON.parse(wasm.catalog()) as Catalog;
  return catalogCache;
}

export interface Fresh {
  input: AnyInput;
  options: Options;
}

export function blank(module: ModuleKey): Fresh {
  return unwrap(call<Fresh>(wasm.blank(module)));
}

export function example(module: ModuleKey): Fresh {
  return unwrap(call<Fresh>(wasm.example(module)));
}

export function fit<T extends AnyInput>(module: ModuleKey, input: T): T {
  const r = call<T>(wasm.fit(module, JSON.stringify(input)));
  return r.ok ? r.value : input;
}

export function setCount<T extends AnyInput>(module: ModuleKey, input: T, count: number): T {
  const r = call<T>(wasm.set_count(module, JSON.stringify(input), count));
  return r.ok ? r.value : input;
}

export function solve(module: ModuleKey, input: AnyInput, options: Options): Result<Outcome> {
  return call<Outcome>(wasm.solve(module, JSON.stringify(input), JSON.stringify(options)));
}

export function lpIterations(input: LpInput, options: Options): Result<{ iterations: Outcome['iterations']; notice: string | null }> {
  return call(wasm.lp_iterations(JSON.stringify(input), JSON.stringify(options)));
}

export function lpGraph(input: LpInput): Result<LpGraph> {
  return call<LpGraph>(wasm.lp_graph(JSON.stringify(input)));
}

export function lpPath(input: LpInput): Result<LpPath> {
  return call<LpPath>(wasm.lp_path(JSON.stringify(input)));
}

export function gameGraph(input: GameInput): Result<GameGraph> {
  return call<GameGraph>(wasm.game_graph(JSON.stringify(input)));
}

export function networkArcs(input: NetInput): Result<NetworkArcs> {
  return call<NetworkArcs>(wasm.network_arcs(JSON.stringify(input)));
}

export function delayMessage(activities: ActivitySchedule[], index: number, delay: number): string {
  return wasm.delay_message(JSON.stringify(activities), index, delay);
}

export function deadlineProbability(mean: number, sd: number, deadline: string): string {
  return wasm.deadline_probability(mean, sd, deadline);
}

export function setFormat(scientific: boolean, width: number, decimals: number): void {
  wasm.set_format(scientific, width, decimals);
}

export function formatNum(v: number): string {
  return wasm.format_num(v);
}

export function openFile(text: string): Result<{ module: ModuleKey; data: AnyInput }> {
  return call(wasm.open_file(text));
}

export function saveFile(module: ModuleKey, data: AnyInput, legacy: boolean): Result<string> {
  return call<string>(wasm.save_file(module, JSON.stringify(data), legacy));
}
