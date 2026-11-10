import { useStudio } from '../store';
import { MoonIcon, MotionIcon, SunIcon } from './icons';
import { useEffectiveTheme, useReducedMotion } from './useDisplay';

export function DisplayToggles() {
  const theme = useEffectiveTheme();
  const reduced = useReducedMotion();
  const setTheme = useStudio((s) => s.setTheme);
  const setReduceMotion = useStudio((s) => s.setReduceMotion);
  const themeLabel = theme === 'dark' ? 'Usar tema claro' : 'Usar tema escuro';
  const motionLabel = reduced ? 'Ativar animações' : 'Reduzir animações';
  return (
    <div className="display-toggles">
      <button type="button" className="icon-btn" onClick={() => setTheme(theme === 'dark' ? 'light' : 'dark')} aria-label={themeLabel} title={themeLabel}>
        {theme === 'dark' ? <SunIcon /> : <MoonIcon />}
      </button>
      <button type="button" className="icon-btn" onClick={() => setReduceMotion(!reduced)} aria-label={motionLabel} title={motionLabel} aria-pressed={reduced}>
        <MotionIcon off={reduced} />
      </button>
    </div>
  );
}
