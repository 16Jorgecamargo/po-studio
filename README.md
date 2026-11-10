# PO Studio

Pesquisa operacional passo a passo: sistemas lineares, inversão de matriz, programação linear (simplex com folgas, M grande, duas fases e dual, análise de sensibilidade e método gráfico), programação inteira (Branch & Bound), transporte, árvore geradora mínima, caminho mais curto, fluxo máximo, CPM, PERT, filas e jogos de soma zero.

O PO Studio existe em duas formas, com o mesmo motor de cálculo:

- **App de computador** (Windows, macOS e Linux), feito com [egui](https://github.com/emilk/egui).
- **Site**, que roda o mesmo núcleo compilado para WebAssembly. Nada é enviado a servidor: o cálculo acontece no navegador.

As duas interfaces são diferentes de propósito, mas fazem as mesmas coisas: abrir e salvar problemas (`.json` e o `.txt` do programa original), exportar relatório, formato numérico, iterações passo a passo, gráficos e tema claro/escuro.

## Baixar

Os instaladores ficam em [Releases](https://github.com/16Jorgecamargo/po-studio/releases).

| Sistema | Arquivo | Como instalar |
|---|---|---|
| macOS (Apple Silicon e Intel) | `PO-Studio-<versão>-macos.dmg` | Abrir o .dmg e arrastar o PO Studio para Aplicativos. Na primeira vez, clicar com o botão direito no app e escolher Abrir. |
| Windows | `PO-Studio-<versão>-windows-setup.exe` | Executar e seguir o assistente. Também há uma versão portátil `...-windows-portable.exe`. |
| Linux (Debian/Ubuntu) | `po-studio_<versão>_amd64.deb` | Dar dois cliques ou rodar `sudo apt install ./po-studio_<versão>_amd64.deb`. |
| Linux (qualquer distribuição) | `PO-Studio-<versão>-linux-x86_64.tar.gz` | Extrair e rodar `./install.sh`. |

## Estrutura

```
crates/po-core   núcleo de cálculo em Rust, sem interface (solvers, relatórios, formato do programa original)
crates/po-wasm   ligação do núcleo com o navegador (wasm-bindgen)
src/             app de computador (egui)
web/             site (React + TypeScript + Vite)
web/src/wasm     núcleo já compilado para WebAssembly, versionado para a Vercel não precisar de Rust
packaging/       scripts dos instaladores
```

## Desenvolvimento

App de computador:

```
cargo run --release
```

Testes do núcleo:

```
cargo test -p po-core
```

Site:

```
cd web
npm install
npm run dev
```

Depois de mudar qualquer coisa em `crates/`, gere o WebAssembly de novo e faça commit do resultado (precisa de [wasm-pack](https://rustwasm.github.io/wasm-pack/)):

```
scripts/build-wasm.sh
```

A integração contínua confere se `web/src/wasm` está em dia com `crates/` (`scripts/check-wasm.sh`).

## Publicar o site na Vercel

1. Importe este repositório na Vercel.
2. Deixe a pasta raiz como está. O `vercel.json` já define instalação, build (`web/dist`) e o redirecionamento das rotas do app.

## Gerar os instaladores

Crie uma tag `v<versão>` (por exemplo `v1.0.1`) e envie para o GitHub. O fluxo `installers` gera os instaladores dos três sistemas e anexa a uma release. Detalhes em [packaging/README.md](packaging/README.md).

## Licença

[MIT](LICENSE)
