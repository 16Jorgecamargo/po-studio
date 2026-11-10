import { useEffect } from 'react';
import * as engine from './engine';
import { Landing } from './landing/Landing';
import { navigate, usePath } from './router';
import { useStudio } from './store';
import type { ModuleKey } from './types';
import { useApplyDisplay } from './ui/useDisplay';
import { Workbench } from './workbench/Workbench';

export function resolve(path: string): { page: 'landing' | 'app' | 'missing'; module: ModuleKey | null } {
  const clean = path.replace(/\/+$/, '') || '/';
  if (clean === '/') return { page: 'landing', module: null };
  if (clean === '/app') return { page: 'app', module: null };
  const m = clean.match(/^\/app\/([a-z_]+)$/);
  if (m && engine.catalog().modules.some((x) => x.key === m[1])) return { page: 'app', module: m[1] as ModuleKey };
  return { page: 'missing', module: null };
}

function Missing() {
  useEffect(() => {
    document.title = 'Página não encontrada · PO Studio';
  }, []);
  return (
    <main className="missing">
      <div className="sheet">
        <h1>Esta página não existe</h1>
        <p>O endereço pode ter mudado. Os módulos estão todos na lista do app.</p>
        <div className="hero-actions">
          <button type="button" className="btn btn--primary" onClick={() => navigate('/app')}>
            Ver os módulos
          </button>
          <button type="button" className="btn" onClick={() => navigate('/')}>
            Ir para o início
          </button>
        </div>
      </div>
    </main>
  );
}

export function App() {
  useApplyDisplay();
  const path = usePath();
  const route = resolve(path);
  const modules = useStudio((s) => s.modules);
  const ready = route.module ? !!modules[route.module] : true;
  useEffect(() => {
    if (route.module && !ready) useStudio.getState().ensure(route.module);
  }, [route.module, ready]);
  if (route.page === 'landing') return <Landing />;
  if (route.page === 'missing') return <Missing />;
  if (!ready) return null;
  return <Workbench module={route.module} />;
}
