import * as engine from '../engine';
import { useStudio } from '../store';
import { Link } from '../ui/Link';
import type { ModuleInfo } from '../types';

export function ModuleIndex({ headingLevel = 2 }: { headingLevel?: 2 | 3 }) {
  const modules = engine.catalog().modules;
  const groups: { name: string; items: ModuleInfo[] }[] = [];
  modules.forEach((m) => {
    const g = groups.find((x) => x.name === m.group);
    if (g) g.items.push(m);
    else groups.push({ name: m.group, items: [m] });
  });
  const H = headingLevel === 2 ? 'h2' : 'h3';
  return (
    <div className="module-index">
      {groups.map((g) => (
        <section key={g.name} className="module-group" aria-label={g.name}>
          <H className="module-group-name">{g.name}</H>
          <ul>
            {g.items.map((m) => (
              <li key={m.key}>
                <Link to={`/app/${m.key}`} className="module-link" onClick={() => useStudio.getState().ensure(m.key)}>
                  <span className="module-name">{m.name}</span>
                  <span className="module-blurb">{m.blurb}</span>
                </Link>
              </li>
            ))}
          </ul>
        </section>
      ))}
    </div>
  );
}
