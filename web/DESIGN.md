# PO Studio na web: direção visual

Direção escrita pelo agente (não pelo autor do projeto), a partir do pedido "interface completamente diferente do app de desktop, mas com todas as funções". Revise à vontade.

## Ideia

Uma folha de resolução corrigida à mão. O enunciado fica a lápis (grafite), as respostas aparecem em caneta azul, o que sai da base e o que é crítico aparece em caneta vermelha, e a coluna que entra recebe marca-texto verde. O pivô de cada quadro é circulado à mão: esse círculo é a assinatura visual do site.

O app de desktop usa o "Quadro do simplex" (papel esverdeado, tinta verde-escura, amarelo, IBM Plex, barra lateral, grade de módulos, abas). A web não repete nada disso: folhas de papel frio sobre uma mesa cinza, barra superior com seletor de módulo, enunciado e solução lado a lado.

## Cores

| Papel | Claro | Escuro ("mesa à noite") |
|---|---|---|
| Mesa | `#E4E8EE` | `#121519` |
| Folha | `#FBFCFD` | `#1B1F26` |
| Grafite (texto) | `#1E232B` | `#E3E7ED` |
| Grafite suave | `#4A5261` | `#AFB6C2` |
| Grafite fraco | `#5A6271` | `#8C95A3` |
| Pauta (linhas) | `#D6DBE3` | `#2D333D` |
| Caneta azul (respostas, foco, pivô) | `#1D3FBF` | `#93A8FF` |
| Caneta vermelha (sai, crítico, erro) | `#C42B27` | `#FF8A80` |
| Marca-texto verde (entra) | `#CDEFB9` | `#2C4A26` |

O tema escuro tem passos próprios, não é inversão automática.

Séries dos gráficos (ordem fixa, validada com o validador do skill dataviz):
claro `#eb6834 #1baf7a #eda100 #e87ba4 #008300 #6250d6`, escuro `#d95926 #199e70 #c98500 #d55181 #008300 #9085e9`.
Azul e vermelho ficam fora das séries porque já significam resposta e erro. No claro, três cores ficam abaixo de 3:1 contra a folha, então toda reta leva rótulo direto e legenda.

## Tipografia

- Atkinson Hyperlegible Next (variável) para a interface: feita para leitura, combina com material didático.
- Atkinson Hyperlegible Mono para números de tabelas e campos: dígitos alinhados em coluna e 0/O, 1/l bem distintos.
- Títulos em peso 700 com espaçamento apertado; rótulos em caixa normal, nunca em maiúsculas espaçadas.

## Estrutura

- Página inicial: título, gráfico vivo do simplex percorrendo os vértices (calculado pelo mesmo motor em WebAssembly), índice dos doze módulos agrupados por área, downloads por sistema e a parte de código aberto.
- App: barra superior (marca, seletor de módulo com Ctrl/⌘ K, menu Arquivo, Formato, tema, animações), folha do enunciado à esquerda e folha da solução à direita. Abaixo de 1100 px as folhas empilham com um seletor Enunciado/Solução.
- Passo a passo: tocador de iterações com marcas por iteração, Anterior/Próxima/Todas/Reiniciar e setas do teclado.

## Movimento

Nível 2. Só três coisas se movem: o círculo do pivô e o do ótimo são "desenhados", o simplex anda pelos vértices na página inicial e os gráficos traçam as retas uma vez ao abrir. Tudo respeita `prefers-reduced-motion` e o botão de reduzir animações.

## Mostradores

ENERGIA 2 · RITMO 2 · MOVIMENTO 2.
