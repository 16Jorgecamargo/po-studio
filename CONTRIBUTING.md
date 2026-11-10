# Como contribuir

Obrigado pelo interesse no PO Studio. Issues e pull requests podem ser em português ou em inglês.

## Relatar um erro

Abra uma [issue](https://github.com/16Jorgecamargo/po-studio/issues/new/choose) usando o modelo "Erro". Ajuda muito incluir:

- o módulo (programação linear, transporte, filas...);
- os dados do problema, de preferência o arquivo `.json` salvo pelo próprio PO Studio;
- o resultado que apareceu e o resultado esperado, com a fonte (livro, lista de exercícios, outro software);
- se aconteceu no site ou no app de computador, e em qual sistema.

## Preparar o ambiente

Precisa de [Rust](https://rustup.rs/) estável, [Node.js](https://nodejs.org/) 22 e, se for mexer no núcleo, [wasm-pack](https://rustwasm.github.io/wasm-pack/).

```
cargo run --release        # app de computador
cargo test -p po-core      # testes do núcleo
cd web && npm install && npm run dev   # site
```

## Onde fica cada coisa

- `crates/po-core`: todos os cálculos. Mudanças de resultado começam aqui, sempre com teste em `crates/po-core/src/tests.rs`.
- `src/`: interface do app de computador (egui).
- `web/`: interface do site (React + TypeScript).
- `crates/po-wasm`: ponte entre o núcleo e o site.

Se mudar qualquer arquivo em `crates/`, rode `scripts/build-wasm.sh` e inclua `web/src/wasm` no commit. A CI falha se o WebAssembly estiver desatualizado.

## Estilo do código

- Nomes de variáveis, funções e arquivos em inglês. Textos da interface em português.
- O código não usa comentários: prefira nomes claros e funções pequenas.
- Siga a formatação dos arquivos vizinhos.
- Rode `cargo test -p po-core` antes de abrir o pull request. No site, `npm run build --prefix web` precisa passar.

## Pull requests

1. Crie um branch a partir de `main`.
2. Faça mudanças pequenas e focadas, uma ideia por pull request.
3. Descreva o que mudou e como conferiu (teste novo, exemplo do livro, captura de tela).

Ao enviar uma contribuição, você concorda que ela seja licenciada sob MIT ou Apache 2.0, como o resto do projeto.

## Conduta

Este projeto segue o [Código de Conduta](CODE_OF_CONDUCT.md). Ao participar, você concorda em respeitá-lo.
