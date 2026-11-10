import * as engine from '../engine';
import { navigate } from '../router';
import { useStudio } from '../store';
import type { ModuleKey } from '../types';

interface WritableLike {
  write: (data: string) => Promise<void>;
  close: () => Promise<void>;
}

interface FileHandleLike {
  name: string;
  getFile: () => Promise<File>;
  createWritable: () => Promise<WritableLike>;
}

interface PickerWindow {
  showSaveFilePicker?: (opts: unknown) => Promise<FileHandleLike>;
  showOpenFilePicker?: (opts: unknown) => Promise<FileHandleLike[]>;
}

const handles = new Map<ModuleKey, FileHandleLike>();

const JSON_TYPE = { description: 'Problema PO Studio', accept: { 'application/json': ['.json'] } };
const TXT_TYPE = { description: 'Arquivo do programa original', accept: { 'text/plain': ['.txt'] } };

function picker(): PickerWindow {
  return window as unknown as PickerWindow;
}

function isAbort(e: unknown): boolean {
  return e instanceof DOMException && e.name === 'AbortError';
}

function download(name: string, text: string) {
  const blob = new Blob([text], { type: name.endsWith('.json') ? 'application/json' : 'text/plain;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = name;
  document.body.appendChild(a);
  a.click();
  a.remove();
  window.setTimeout(() => URL.revokeObjectURL(url), 1000);
}

function content(key: ModuleKey, legacy: boolean): string | null {
  const m = useStudio.getState().modules[key];
  if (!m) return null;
  const r = engine.saveFile(key, m.input, legacy);
  if (!r.ok) {
    useStudio.getState().setStatus(r.error, 'error');
    return null;
  }
  return r.value;
}

async function writeHandle(handle: FileHandleLike, text: string) {
  const w = await handle.createWritable();
  await w.write(text);
  await w.close();
}

export async function saveAs(key: ModuleKey, legacy = false): Promise<void> {
  const st = useStudio.getState();
  const p = picker();
  if (p.showSaveFilePicker) {
    try {
      const handle = await p.showSaveFilePicker({
        suggestedName: `${key}.${legacy ? 'txt' : 'json'}`,
        types: legacy ? [TXT_TYPE] : [JSON_TYPE, TXT_TYPE],
      });
      const text = content(key, handle.name.toLowerCase().endsWith('.txt'));
      if (text === null) return;
      await writeHandle(handle, text);
      handles.set(key, handle);
      st.setStatus(`Salvo em ${handle.name}`);
    } catch (e) {
      if (isAbort(e)) st.setStatus('Você não indicou um nome de arquivo.', 'error');
      else st.setStatus(`Não foi possível salvar: ${(e as Error).message}`, 'error');
    }
    return;
  }
  const text = content(key, legacy);
  if (text === null) return;
  const name = `${key}.${legacy ? 'txt' : 'json'}`;
  download(name, text);
  st.setStatus(`Arquivo baixado: ${name}`);
}

export async function save(key: ModuleKey): Promise<void> {
  const handle = handles.get(key);
  if (!handle) return saveAs(key);
  const text = content(key, handle.name.toLowerCase().endsWith('.txt'));
  if (text === null) return;
  try {
    await writeHandle(handle, text);
    useStudio.getState().setStatus(`Salvo em ${handle.name}`);
  } catch (e) {
    useStudio.getState().setStatus(`Não foi possível salvar: ${(e as Error).message}`, 'error');
  }
}

export function openText(text: string, name: string, handle?: FileHandleLike): void {
  const st = useStudio.getState();
  const r = engine.openFile(text);
  if (!r.ok) {
    st.setStatus(r.error, 'error');
    return;
  }
  const { module, data } = r.value;
  st.ensure(module);
  st.loadData(module, data);
  if (handle) handles.set(module, handle);
  else handles.delete(module);
  navigate(`/app/${module}`);
  st.setStatus(`Aberto: ${name}`);
}

export async function openFileDialog(): Promise<void> {
  const st = useStudio.getState();
  const p = picker();
  if (p.showOpenFilePicker) {
    try {
      const [handle] = await p.showOpenFilePicker({
        types: [{ description: 'Problemas', accept: { 'application/json': ['.json'], 'text/plain': ['.txt'] } }, JSON_TYPE, TXT_TYPE],
        multiple: false,
      });
      const file = await handle.getFile();
      openText(await file.text(), file.name, handle);
    } catch (e) {
      if (!isAbort(e)) st.setStatus(`Não foi possível abrir: ${(e as Error).message}`, 'error');
    }
    return;
  }
  const input = document.createElement('input');
  input.type = 'file';
  input.accept = '.json,.txt,application/json,text/plain';
  input.onchange = async () => {
    const file = input.files?.[0];
    if (!file) return;
    try {
      openText(await file.text(), file.name);
    } catch (e) {
      st.setStatus(`Não foi possível abrir: ${(e as Error).message}`, 'error');
    }
  };
  input.click();
}

export async function openDropped(file: File): Promise<void> {
  try {
    openText(await file.text(), file.name);
  } catch (e) {
    useStudio.getState().setStatus(`Não foi possível abrir: ${(e as Error).message}`, 'error');
  }
}

export async function exportReport(key: ModuleKey): Promise<void> {
  const st = useStudio.getState();
  const m = st.modules[key];
  const report = m?.outcome?.report;
  if (!report) {
    st.setStatus('Resolva o problema antes de exportar o relatório.', 'error');
    return;
  }
  const name = `${key}_relatorio.txt`;
  const p = picker();
  if (p.showSaveFilePicker) {
    try {
      const handle = await p.showSaveFilePicker({ suggestedName: name, types: [{ description: 'Texto', accept: { 'text/plain': ['.txt'] } }] });
      await writeHandle(handle, report);
      st.setStatus(`Relatório salvo em ${handle.name}`);
    } catch (e) {
      if (!isAbort(e)) st.setStatus(`Não foi possível salvar: ${(e as Error).message}`, 'error');
    }
    return;
  }
  download(name, report);
  st.setStatus(`Relatório baixado: ${name}`);
}

export function forget(key: ModuleKey): void {
  handles.delete(key);
}
