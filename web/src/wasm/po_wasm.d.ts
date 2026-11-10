/* tslint:disable */
/* eslint-disable */

export function blank(module: string): string;

export function catalog(): string;

export function deadline_probability(mean: number, sd: number, deadline: string): string;

export function delay_message(activities: string, index: number, delay: number): string;

export function example(module: string): string;

export function fit(module: string, input: string): string;

export function format_num(v: number): string;

export function game_graph(input: string): string;

export function lp_graph(input: string): string;

export function lp_iterations(input: string, options: string): string;

export function lp_path(input: string): string;

export function network_arcs(input: string): string;

export function open_file(text: string): string;

export function save_file(module: string, data: string, legacy: boolean): string;

export function set_count(module: string, input: string, count: number): string;

export function set_format(scientific: boolean, width: number, decimals: number): void;

export function solve(module: string, input: string, options: string): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly blank: (a: number, b: number) => [number, number];
    readonly catalog: () => [number, number];
    readonly deadline_probability: (a: number, b: number, c: number, d: number) => [number, number];
    readonly delay_message: (a: number, b: number, c: number, d: number) => [number, number];
    readonly example: (a: number, b: number) => [number, number];
    readonly fit: (a: number, b: number, c: number, d: number) => [number, number];
    readonly format_num: (a: number) => [number, number];
    readonly game_graph: (a: number, b: number) => [number, number];
    readonly lp_graph: (a: number, b: number) => [number, number];
    readonly lp_iterations: (a: number, b: number, c: number, d: number) => [number, number];
    readonly lp_path: (a: number, b: number) => [number, number];
    readonly network_arcs: (a: number, b: number) => [number, number];
    readonly open_file: (a: number, b: number) => [number, number];
    readonly save_file: (a: number, b: number, c: number, d: number, e: number) => [number, number];
    readonly set_count: (a: number, b: number, c: number, d: number, e: number) => [number, number];
    readonly set_format: (a: number, b: number, c: number) => void;
    readonly solve: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
