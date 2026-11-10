import { create } from 'zustand';
import * as engine from './engine';
import { readJson, writeJson } from './storage';
import type { AnyInput, LpInput, ModuleKey, Options, Outcome } from './types';

export type ViewKey = 'results' | 'iterations' | 'graph';
export type Theme = 'light' | 'dark';

export interface NumberFormat {
  scientific: boolean;
  width: number;
  decimals: number;
}

export interface ModuleState {
  input: AnyInput;
  options: Options;
  outcome: Outcome | null;
  error: string | null;
  notice: string | null;
  view: ViewKey;
  shown: number;
  stale: boolean;
  hiddenLines: number[];
  showPath: boolean;
  bbSelected: number | null;
  delayAct: number;
  delay: number;
  deadline: string;
  queueSelected: number;
  solvedAt: number;
}

export interface Status {
  text: string;
  tone: 'ok' | 'error';
  at: number;
}

interface Settings {
  theme: Theme | null;
  reduceMotion: boolean | null;
  format: NumberFormat;
}

interface Drafts {
  [key: string]: { input: AnyInput; options: Options };
}

interface StudioState extends Settings {
  modules: Partial<Record<ModuleKey, ModuleState>>;
  status: Status | null;
  ensure: (key: ModuleKey) => ModuleState;
  patch: (key: ModuleKey, part: Partial<ModuleState>) => void;
  setInput: (key: ModuleKey, input: AnyInput) => void;
  setOptions: (key: ModuleKey, options: Options) => void;
  solve: (key: ModuleKey) => boolean;
  rebuildLpIterations: (key: ModuleKey, options: Options) => void;
  newProblem: (key: ModuleKey) => void;
  loadExample: (key: ModuleKey) => void;
  loadData: (key: ModuleKey, input: AnyInput) => void;
  setStatus: (text: string, tone?: Status['tone']) => void;
  clearStatus: () => void;
  setTheme: (theme: Theme) => void;
  setReduceMotion: (value: boolean) => void;
  setFormat: (format: NumberFormat) => void;
}

const SETTINGS_KEY = 'po-studio:settings';
const DRAFTS_KEY = 'po-studio:drafts';

const DEFAULT_FORMAT: NumberFormat = { scientific: false, width: 5, decimals: 2 };

function fresh(input: AnyInput, options: Options): ModuleState {
  return {
    input,
    options,
    outcome: null,
    error: null,
    notice: null,
    view: 'results',
    shown: 1,
    stale: false,
    hiddenLines: [],
    showPath: true,
    bbSelected: null,
    delayAct: 0,
    delay: 0,
    deadline: '',
    queueSelected: 0,
    solvedAt: 0,
  };
}

function initialModule(key: ModuleKey): ModuleState {
  const drafts = readJson<Drafts>(DRAFTS_KEY);
  const saved = drafts?.[key];
  if (saved) {
    const input = engine.fit(key, saved.input);
    return fresh(input, { ...engine.blank(key).options, ...saved.options });
  }
  const ex = engine.example(key);
  return fresh(ex.input, ex.options);
}

const savedSettings = readJson<Partial<Settings>>(SETTINGS_KEY);

function applyFormat(f: NumberFormat) {
  engine.setFormat(f.scientific, f.width, f.decimals);
}

export const useStudio = create<StudioState>((set, get) => ({
  modules: {},
  status: null,
  theme: savedSettings?.theme ?? null,
  reduceMotion: savedSettings?.reduceMotion ?? null,
  format: { ...DEFAULT_FORMAT, ...(savedSettings?.format ?? {}) },

  ensure(key) {
    const existing = get().modules[key];
    if (existing) return existing;
    const m = initialModule(key);
    set((s) => ({ modules: { ...s.modules, [key]: m } }));
    return m;
  },

  patch(key, part) {
    set((s) => {
      const cur = s.modules[key] ?? initialModule(key);
      return { modules: { ...s.modules, [key]: { ...cur, ...part } } };
    });
  },

  setInput(key, input) {
    const cur = get().ensure(key);
    get().patch(key, { input, stale: cur.outcome !== null });
  },

  setOptions(key, options) {
    const cur = get().ensure(key);
    get().patch(key, { options, stale: cur.outcome !== null });
  },

  solve(key) {
    const cur = get().ensure(key);
    applyFormat(get().format);
    const r = engine.solve(key, cur.input, cur.options);
    if (!r.ok) {
      get().patch(key, { error: r.error, outcome: null, notice: null, stale: false });
      return false;
    }
    const out = r.value;
    get().patch(key, {
      outcome: out,
      error: null,
      notice: out.notice,
      shown: 1,
      stale: false,
      bbSelected: null,
      delayAct: 0,
      delay: 0,
      queueSelected: 0,
      solvedAt: Date.now(),
    });
    return true;
  },

  rebuildLpIterations(key, options) {
    const cur = get().ensure(key);
    applyFormat(get().format);
    const r = engine.lpIterations(cur.input as LpInput, options);
    const outcome = cur.outcome;
    if (!outcome) {
      get().patch(key, { options });
      return;
    }
    if (r.ok) {
      get().patch(key, {
        options,
        notice: r.value.notice,
        shown: 1,
        outcome: { ...outcome, iterations: r.value.iterations, iterError: null },
      });
    } else {
      get().patch(key, { options, shown: 1, outcome: { ...outcome, iterations: [], iterError: r.error } });
    }
  },

  newProblem(key) {
    const b = engine.blank(key);
    get().patch(key, fresh(b.input, b.options));
    get().clearStatus();
  },

  loadExample(key) {
    const ex = engine.example(key);
    get().patch(key, fresh(ex.input, ex.options));
    get().setStatus('Exemplo carregado. Clique em Resolver.');
  },

  loadData(key, input) {
    const cur = get().ensure(key);
    get().patch(key, fresh(engine.fit(key, input), cur.options));
  },

  setStatus(text, tone = 'ok') {
    set({ status: { text, tone, at: Date.now() } });
  },

  clearStatus() {
    set({ status: null });
  },

  setTheme(theme) {
    set({ theme });
  },

  setReduceMotion(value) {
    set({ reduceMotion: value });
  },

  setFormat(format) {
    set({ format });
    applyFormat(format);
    const mods = get().modules;
    (Object.keys(mods) as ModuleKey[]).forEach((k) => {
      const m = mods[k];
      if (m?.outcome && !m.stale) {
        const view = m.view;
        const shown = m.shown;
        if (get().solve(k)) get().patch(k, { view, shown });
      }
    });
  },
}));

applyFormat(useStudio.getState().format);

let timer: number | undefined;
useStudio.subscribe((s) => {
  window.clearTimeout(timer);
  timer = window.setTimeout(() => {
    writeJson(SETTINGS_KEY, { theme: s.theme, reduceMotion: s.reduceMotion, format: s.format });
    const drafts: Drafts = readJson<Drafts>(DRAFTS_KEY) ?? {};
    (Object.keys(s.modules) as ModuleKey[]).forEach((k) => {
      const m = s.modules[k];
      if (m) drafts[k] = { input: m.input, options: m.options };
    });
    writeJson(DRAFTS_KEY, drafts);
  }, 300);
});

export function useModule(key: ModuleKey): ModuleState {
  return useStudio((s) => s.modules[key]) as ModuleState;
}
