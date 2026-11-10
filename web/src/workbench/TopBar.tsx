import { useEffect } from 'react';
import { useStudio } from '../store';
import type { ModuleKey } from '../types';
import { DisplayToggles } from '../ui/DisplayToggles';
import { Link } from '../ui/Link';
import { FileMenu } from './FileMenu';
import { FormatMenu } from './FormatMenu';
import { ModuleSwitcher } from './ModuleSwitcher';

export function StatusLine() {
  const status = useStudio((s) => s.status);
  const clear = useStudio((s) => s.clearStatus);
  useEffect(() => {
    if (!status || status.tone === 'error') return;
    const t = window.setTimeout(clear, 6000);
    return () => window.clearTimeout(t);
  }, [status, clear]);
  return (
    <div className="status-slot" aria-live="polite">
      {status ? (
        <p className={`status status--${status.tone}`} key={status.at}>
          <span>{status.text}</span>
          <button type="button" className="status-close" onClick={clear} aria-label="Fechar aviso">
            ×
          </button>
        </p>
      ) : null}
    </div>
  );
}

export function TopBar({ module }: { module: ModuleKey | null }) {
  return (
    <header className="topbar">
      <div className="topbar-row">
        <Link to="/" className="wordmark" aria-label="PO Studio, página inicial">
          PO Studio
        </Link>
        <ModuleSwitcher current={module} />
        {module ? (
          <nav className="bar-menus" aria-label="Ações">
            <FileMenu module={module} />
            <FormatMenu />
          </nav>
        ) : (
          <span className="bar-spacer" />
        )}
        <DisplayToggles />
      </div>
      <StatusLine />
    </header>
  );
}
