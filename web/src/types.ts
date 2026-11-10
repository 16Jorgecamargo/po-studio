export type CellStyle = 'normal' | 'head' | 'green' | 'red' | 'pivot' | 'bold' | 'muted' | 'red_text';

export interface Cell {
  text: string;
  style: CellStyle;
}

export interface Section {
  title: string;
  subtitle: string;
  headers: string[];
  rows: Cell[][];
  notes: string[];
  lines: string[];
}

export type ModuleKey =
  | 'linear_equations'
  | 'matrix_inverse'
  | 'linear_programming'
  | 'integer_programming'
  | 'transportation'
  | 'minimal_spanning_tree'
  | 'shortest_route'
  | 'maximal_flow'
  | 'cpm'
  | 'pert'
  | 'queuing'
  | 'zero_sum_game';

export interface ModuleInfo {
  key: ModuleKey;
  group: string;
  name: string;
  blurb: string;
  hint: string;
}

export interface Choice {
  key: string;
  label: string;
}

export interface Catalog {
  modules: ModuleInfo[];
  maxSize: number;
  queueHeaders: string[];
  cpmHeaders: string[];
  pertHeaders: string[];
  iterMethods: Choice[];
  startMethods: Choice[];
}

export interface AlgebraInput {
  title: string;
  n: number;
  a: string[][];
  b: string[];
}

export interface LpInput {
  title: string;
  maximize: boolean;
  nvars: number;
  ncons: number;
  var_names: string[];
  obj: string[];
  con_names: string[];
  a: string[][];
  rel: string[];
  rhs: string[];
  lower: string[];
  upper: string[];
  unrestricted: boolean[];
  integer: boolean[];
}

export interface TransportInput {
  title: string;
  ns: number;
  nd: number;
  sources: string[];
  dests: string[];
  cost: string[][];
  supply: string[];
  demand: string[];
}

export interface NetInput {
  title: string;
  n: number;
  names: string[];
  matrix: string[][];
}

export interface RowsInput {
  title: string;
  rows: string[][];
}

export interface GameInput {
  title: string;
  m: number;
  n: number;
  payoff: string[][];
}

export type AnyInput = AlgebraInput | LpInput | TransportInput | NetInput | RowsInput | GameInput;

export interface Options {
  method?: string;
  big_m?: string;
  algo?: string;
  start?: number;
  target?: number;
}

export interface BbNodeView {
  id: number;
  parent: number | null;
  depth: number;
  branch: string;
  z: number | null;
  zText: string | null;
  status: 'infeasible' | 'unbounded' | 'integer' | 'fathomed' | 'branched';
  statusText: string;
  pathText: string;
  solutionText: string | null;
}

export interface ActivitySchedule {
  name: string;
  from: number;
  to: number;
  duration: number;
  es: number;
  ef: number;
  ls: number;
  lf: number;
  total_float: number;
  free_float: number;
  critical: boolean;
}

export interface OutcomeView {
  tree?: { nodes: BbNodeView[]; best: number | null; aborted: boolean };
  activities?: ActivitySchedule[];
  duration?: number;
  pathStats?: { mean: number; sd: number } | null;
  scenarios?: string[];
  highlight?: { from: number; to: number; label: string }[];
  start?: number;
  target?: number;
}

export interface Outcome {
  results: Section[];
  iterations: Section[];
  notice: string | null;
  report: string;
  view: OutcomeView;
  iterError?: string | null;
}

export interface LpGraph {
  xmax: number;
  ymax: number;
  region: [number, number][];
  status: 'optimal' | 'unbounded' | 'infeasible';
  optimum: [number, number] | null;
  z: number | null;
  c: [number, number];
  summary: string;
  marker: string | null;
  xLabel: string;
  yLabel: string;
  lines: { label: string; tag: string; segment: [[number, number], [number, number]] | null }[];
}

export interface LpPath {
  points: { x: number[]; z: number; phase: number; iteration: number }[];
}

export interface GameGraph {
  row_player: boolean;
  who: string;
  lines: [number, number][];
  best_p: number;
  best_v: number;
  ymin: number;
  ymax: number;
  envelope: [number, number][];
  heading: string;
  optimum: string;
  explanation: string;
  marker: string;
  labels: string[];
  x_label: string;
}

export interface NetworkArcs {
  names: string[];
  arcs: { from: number; to: number; value: number; label: string }[];
}

export type Result<T> = { ok: true; value: T } | { ok: false; error: string; info?: boolean };
