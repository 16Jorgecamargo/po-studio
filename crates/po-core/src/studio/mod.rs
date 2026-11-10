pub mod algebra;
pub mod game;
pub mod lp;
pub mod network;
pub mod project;
pub mod queue;
pub mod transport;

use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct ModuleInfo {
    pub key: &'static str,
    pub group: &'static str,
    pub name: &'static str,
    pub blurb: &'static str,
}

pub const MODULES: [ModuleInfo; 12] = [
    ModuleInfo { key: "linear_equations", group: "Álgebra", name: "Equações lineares", blurb: "Resolve sistemas de equações simultâneas e calcula o determinante." },
    ModuleInfo { key: "matrix_inverse", group: "Álgebra", name: "Inversão de matriz", blurb: "Inversa e determinante de uma matriz quadrada." },
    ModuleInfo { key: "linear_programming", group: "Otimização", name: "Programação linear", blurb: "Simplex (folgas, M grande, duas fases, dual), sensibilidade e método gráfico." },
    ModuleInfo { key: "integer_programming", group: "Otimização", name: "Programação inteira", blurb: "Branch & Bound com árvore de busca interativa." },
    ModuleInfo { key: "transportation", group: "Otimização", name: "Modelo de transporte", blurb: "Noroeste, custo mínimo e Vogel, com quadros de iteração." },
    ModuleInfo { key: "minimal_spanning_tree", group: "Redes", name: "Árvore geradora mínima", blurb: "Conecta todos os nós com o menor comprimento total." },
    ModuleInfo { key: "shortest_route", group: "Redes", name: "Caminho mais curto", blurb: "Algoritmos de Dijkstra e de Floyd." },
    ModuleInfo { key: "maximal_flow", group: "Redes", name: "Fluxo máximo", blurb: "Maior fluxo possível da origem ao destino." },
    ModuleInfo { key: "cpm", group: "Projetos", name: "CPM", blurb: "Caminho crítico, folgas e gráfico de barras." },
    ModuleInfo { key: "pert", group: "Projetos", name: "PERT", blurb: "Durações incertas, média, variância e probabilidade de prazo." },
    ModuleInfo { key: "queuing", group: "Probabilidade", name: "Filas", blurb: "Modelos (M/M/c):(GD/N/K), nascimento e morte puros." },
    ModuleInfo { key: "zero_sum_game", group: "Probabilidade", name: "Jogos de soma zero", blurb: "Estratégias ótimas por PL e solução gráfica." },
];

pub const QUEUE_HINT: &str = "Modelo (M/M/c):(GD/N/K). Use ∞ para fila ou fonte infinita. Nascimento puro: μ = 0 e λ = λt. Morte pura: λ = 0, μ = μt e N = tamanho inicial.";
pub const LP_HINT: &str = "Tamanho máximo: 100 (como no TORA). Deixe em branco para zero; use ∞ ou inf para infinito.";
pub const PROJECT_HINT: &str = "Cada atividade liga um nó (evento) a outro. Numere os nós a partir de 1.";
pub const GAME_HINT: &str = "Os pagamentos são do ponto de vista do jogador A (linhas).";
