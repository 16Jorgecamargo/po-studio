import '@fontsource-variable/atkinson-hyperlegible-next';
import '@fontsource-variable/atkinson-hyperlegible-mono';
import './styles/tokens.css';
import './styles/base.css';
import './styles/landing.css';
import './styles/workbench.css';
import './styles/sheet.css';
import './styles/charts.css';
import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { loadEngine } from './engine';

const root = createRoot(document.getElementById('root')!);

function Failure({ message }: { message: string }) {
  return (
    <main className="missing">
      <div className="sheet">
        <h1>O motor de cálculo não carregou</h1>
        <p>O navegador não conseguiu iniciar o WebAssembly: {message}</p>
        <p>Recarregue a página. Se continuar, tente um navegador atualizado ou baixe o app de computador.</p>
      </div>
    </main>
  );
}

loadEngine()
  .then(async () => {
    const [{ App, resolve }, { useStudio }] = await Promise.all([import('./App'), import('./store')]);
    const ensureRoute = () => {
      const r = resolve(window.location.pathname);
      if (r.module) useStudio.getState().ensure(r.module);
    };
    window.addEventListener('popstate', ensureRoute);
    window.addEventListener('po-studio:navigate', ensureRoute);
    ensureRoute();
    root.render(
      <StrictMode>
        <App />
      </StrictMode>,
    );
  })
  .catch((e: unknown) => {
    root.render(<Failure message={e instanceof Error ? e.message : String(e)} />);
  });
