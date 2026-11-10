import type { ViewKey } from '../store';
import type { ModuleKey } from '../types';

export interface ViewDef {
  key: ViewKey;
  label: string;
}

const algebra: ViewDef[] = [
  { key: 'results', label: 'Resultado' },
  { key: 'iterations', label: 'Passos' },
];

const network: ViewDef[] = [
  { key: 'results', label: 'Resultado' },
  { key: 'iterations', label: 'Iterações' },
  { key: 'graph', label: 'Rede' },
];

const project: ViewDef[] = [
  { key: 'results', label: 'Resultado' },
  { key: 'iterations', label: 'Cálculos' },
  { key: 'graph', label: 'Gráfico de barras' },
];

export const VIEWS: Record<ModuleKey, ViewDef[]> = {
  linear_equations: algebra,
  matrix_inverse: algebra,
  linear_programming: [
    { key: 'results', label: 'Resultado' },
    { key: 'iterations', label: 'Iterações' },
    { key: 'graph', label: 'Gráfico' },
  ],
  integer_programming: [
    { key: 'results', label: 'Resultado' },
    { key: 'graph', label: 'Árvore B&B' },
    { key: 'iterations', label: 'Subproblemas' },
  ],
  transportation: [
    { key: 'results', label: 'Resultado' },
    { key: 'iterations', label: 'Iterações' },
  ],
  minimal_spanning_tree: network,
  shortest_route: network,
  maximal_flow: network,
  cpm: project,
  pert: project,
  queuing: [
    { key: 'results', label: 'Comparação' },
    { key: 'iterations', label: 'Probabilidades' },
  ],
  zero_sum_game: [
    { key: 'results', label: 'Resultado' },
    { key: 'graph', label: 'Gráfico' },
  ],
};

export const LIVE_GRAPH: Partial<Record<ModuleKey, boolean>> = {
  linear_programming: true,
  zero_sum_game: true,
  minimal_spanning_tree: true,
  shortest_route: true,
  maximal_flow: true,
};
