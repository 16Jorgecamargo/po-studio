import { useEffect, useMemo } from 'react';
import * as engine from '../engine';
import { REPO_URL, RELEASES_URL } from '../site';
import type { LpInput } from '../types';
import { DisplayToggles } from '../ui/DisplayToggles';
import { Link } from '../ui/Link';
import { SectionView } from '../workbench/results/SectionView';
import { HeroGraph } from './HeroGraph';
import { ModuleIndex } from './ModuleIndex';

const DOWNLOADS = [
  {
    os: 'macOS',
    detail: 'Apple Silicon e Intel',
    file: 'PO-Studio-<versão>-macos.dmg',
    how: 'Abra o .dmg e arraste o PO Studio para Aplicativos. Na primeira vez, clique com o botão direito no app e escolha Abrir.',
  },
  {
    os: 'Windows',
    detail: 'Instalador ou versão portátil',
    file: 'PO-Studio-<versão>-windows-setup.exe',
    how: 'Execute e siga o assistente. Quem não pode instalar programas usa o ...-windows-portable.exe, que roda direto.',
  },
  {
    os: 'Linux',
    detail: 'Debian e Ubuntu',
    file: 'po-studio_<versão>_amd64.deb',
    how: 'Dê dois cliques ou rode sudo apt install ./po-studio_<versão>_amd64.deb.',
  },
  {
    os: 'Linux',
    detail: 'Qualquer distribuição',
    file: 'PO-Studio-<versão>-linux-x86_64.tar.gz',
    how: 'Extraia e rode ./install.sh. Instala só para o seu usuário, sem pedir senha.',
  },
];

function TableauSample() {
  const sample = useMemo(() => {
    const ex = engine.example('linear_programming');
    const r = engine.solve('linear_programming', ex.input as LpInput, ex.options);
    if (!r.ok) return null;
    return r.value.iterations.find((s) => s.rows.some((row) => row.some((c) => c.style === 'pivot'))) ?? null;
  }, []);
  if (!sample) return null;
  return <SectionView section={sample} id="sample" />;
}

export function Landing() {
  useEffect(() => {
    document.title = 'PO Studio: pesquisa operacional passo a passo';
  }, []);

  return (
    <div className="landing">
      <a className="skip-link" href="#main">
        Pular para o conteúdo
      </a>
      <header className="site-header">
        <Link to="/" className="wordmark">
          PO Studio
        </Link>
        <nav className="site-nav" aria-label="Principal">
          <a href="#modulos">Módulos</a>
          <a href="#baixar">Baixar</a>
          <a href={REPO_URL}>Código</a>
        </nav>
        <DisplayToggles />
        <Link to="/app" className="btn btn--primary btn--small header-cta">
          Abrir o app
        </Link>
      </header>

      <main id="main">
        <section className="hero">
          <div className="hero-text">
            <h1>Pesquisa operacional, resolvida passo a passo.</h1>
            <p className="hero-lede">
              Simplex com sensibilidade e método gráfico, Branch &amp; Bound, transporte, redes, CPM e PERT, filas e jogos. Cada quadro, cada pivô e cada iteração ficam à vista.
            </p>
            <div className="hero-actions">
              <Link to="/app/linear_programming" className="btn btn--primary">
                Abrir o app
              </Link>
              <a href="#baixar" className="btn">
                Baixar para o computador
              </a>
            </div>
            <p className="hero-note">Funciona no navegador, sem cadastro. O app de computador usa o mesmo motor de cálculo, então os números batem.</p>
          </div>
          <HeroGraph />
        </section>

        <section id="modulos" className="band band--modules" aria-labelledby="modulos-t">
          <div className="band-intro">
            <h2 id="modulos-t">Doze módulos, do sistema linear ao jogo de soma zero</h2>
            <p>Cada um abre com um exemplo pronto. Troque os números, clique em Resolver e acompanhe as iterações.</p>
          </div>
          <ModuleIndex headingLevel={3} />
        </section>

        <section className="band band--steps" aria-labelledby="passos-t">
          <div className="steps-text">
            <h2 id="passos-t">A solução vem com o rascunho</h2>
            <p>
              Nas iterações do simplex, a coluna que entra recebe marca-texto, a linha que sai fica em vermelho e o pivô é circulado. Avance um quadro por vez, volte, ou mostre todos de uma vez.
            </p>
            <p>Os arquivos do programa original (.txt) abrem aqui e podem ser salvos de volta no mesmo formato.</p>
          </div>
          <div className="steps-sample sheet">
            <TableauSample />
          </div>
        </section>

        <section id="baixar" className="band band--download" aria-labelledby="baixar-t">
          <div className="band-intro">
            <h2 id="baixar-t">Baixar para o computador</h2>
            <p>
              Os instaladores ficam na página de versões do GitHub. O app de computador funciona sem internet e guarda os problemas como arquivos.
            </p>
          </div>
          <ul className="download-list">
            {DOWNLOADS.map((d) => (
              <li key={d.file} className="download-item">
                <div className="download-os">
                  <h3>{d.os}</h3>
                  <p>{d.detail}</p>
                </div>
                <div className="download-how">
                  <p className="download-file">{d.file}</p>
                  <p>{d.how}</p>
                </div>
              </li>
            ))}
          </ul>
          <a className="btn btn--primary" href={RELEASES_URL}>
            Ir para a página de versões
          </a>
        </section>

        <section className="band band--source" aria-labelledby="codigo-t">
          <div className="band-intro">
            <h2 id="codigo-t">Código aberto</h2>
            <p>
              O núcleo de cálculo é escrito em Rust e não depende de interface. O app de computador desenha com egui; este site usa o mesmo núcleo compilado para WebAssembly. Licença MIT.
            </p>
          </div>
          <div className="source-grid">
            <div>
              <h3>Rodar o app de computador</h3>
              <pre className="code">
                <code>cargo run --release</code>
              </pre>
            </div>
            <div>
              <h3>Rodar este site</h3>
              <pre className="code">
                <code>{'cd web\nnpm install\nnpm run dev'}</code>
              </pre>
            </div>
          </div>
          <a className="btn" href={REPO_URL}>
            Ver o repositório no GitHub
          </a>
        </section>
      </main>

      <footer className="site-footer">
        <p>
          PO Studio é código aberto sob a licença MIT. <a href={REPO_URL}>Repositório no GitHub</a>.
        </p>
      </footer>
    </div>
  );
}
