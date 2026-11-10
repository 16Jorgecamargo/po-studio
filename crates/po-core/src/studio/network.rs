use crate::format::{num, parse_num};
use crate::grid::{resize_grid, resize_names};
use crate::network::{dijkstra, floyd, floyd_route, max_flow, minimal_spanning_tree, route, INF};
use crate::report::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetKind {
    Mst,
    Shortest,
    MaxFlow,
}

impl NetKind {
    pub fn key(&self) -> &'static str {
        match self {
            NetKind::Mst => "minimal_spanning_tree",
            NetKind::Shortest => "shortest_route",
            NetKind::MaxFlow => "maximal_flow",
        }
    }

    pub fn hint(&self) -> &'static str {
        match self {
            NetKind::Mst => "Informe a distância entre os nós (basta preencher uma das células i→j ou j→i). Vazio = sem ligação.",
            NetKind::Shortest => "Linha = nó de origem, coluna = nó de destino. Vazio = sem arco.",
            NetKind::MaxFlow => "Capacidade do arco da linha (de) para a coluna (para). Vazio = sem arco.",
        }
    }

    pub fn report_title(&self) -> &'static str {
        match self {
            NetKind::Mst => "ÁRVORE GERADORA MÍNIMA",
            NetKind::Shortest => "CAMINHO MAIS CURTO",
            NetKind::MaxFlow => "FLUXO MÁXIMO",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShortAlgo {
    Dijkstra,
    Floyd,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NetInput {
    pub title: String,
    pub n: usize,
    pub names: Vec<String>,
    pub matrix: Vec<Vec<String>>,
}

impl NetInput {
    pub fn blank(n: usize) -> Self {
        let mut s = Self {
            title: String::new(),
            n,
            names: vec![],
            matrix: vec![],
        };
        s.fit();
        s
    }

    pub fn fit(&mut self) {
        resize_names(&mut self.names, self.n, "N");
        resize_grid(&mut self.matrix, self.n, self.n);
    }

    pub fn example(kind: NetKind) -> (Self, usize, usize) {
        let mut inp = NetInput::blank(6);
        let arcs: &[(usize, usize, &str)] = match kind {
            NetKind::Mst => &[(0, 1, "4"), (0, 2, "3"), (1, 2, "2"), (1, 3, "6"), (2, 3, "5"), (2, 4, "7"), (3, 4, "3"), (3, 5, "4"), (4, 5, "2")],
            NetKind::Shortest => &[(0, 1, "7"), (0, 2, "3"), (1, 3, "2"), (2, 1, "2"), (2, 3, "8"), (2, 4, "6"), (3, 5, "4"), (4, 3, "1"), (4, 5, "7")],
            NetKind::MaxFlow => &[(0, 1, "12"), (0, 2, "9"), (1, 2, "4"), (1, 3, "8"), (2, 4, "10"), (3, 4, "3"), (3, 5, "9"), (4, 5, "11")],
        };
        for &(i, j, v) in arcs {
            inp.matrix[i][j] = v.to_string();
        }
        inp.title = "Rede de exemplo com 6 nós".into();
        (inp, 0, 5)
    }

    pub fn distances(&self) -> Vec<Vec<f64>> {
        let n = self.n;
        (0..n)
            .map(|i| {
                (0..n)
                    .map(|j| {
                        if i == j {
                            INF
                        } else {
                            match parse_num(&self.matrix[i][j]) {
                                Some(v) if v.is_finite() => v,
                                _ => INF,
                            }
                        }
                    })
                    .collect()
            })
            .collect()
    }
}

pub struct NetOutcome {
    pub results: Vec<Section>,
    pub iterations: Vec<Section>,
    pub highlight: Vec<(usize, usize, String)>,
    pub start: usize,
    pub target: usize,
}

pub fn solve(input: &NetInput, kind: NetKind, algo: ShortAlgo, start: usize, target: usize) -> Result<NetOutcome, String> {
    let d = input.distances();
    let names = input.names.clone();
    let n = input.n;
    let start = start.min(n - 1);
    let target = target.min(n - 1);
    let title = title_or_untitled(&input.title);
    let mut highlight = Vec::new();
    let (results, iterations) = match kind {
        NetKind::Mst => {
            let steps = minimal_spanning_tree(&d, start)?;
            let total = steps.last().map(|s| s.total).unwrap_or(0.0);
            let mut h = Section::new("Árvore geradora mínima: resumo").sub(title);
            h.line(format!("Comprimento da árvore geradora mínima = {}", num(total)));
            let mut t = Section::new("Arcos da árvore").headers(&["De", "Para", "Comprimento"]);
            for s in &steps {
                t.row(vec![cs(names[s.from].clone(), CellStyle::Head), c(names[s.to].clone()), c(num(s.length))]);
                highlight.push((s.from, s.to, num(s.length)));
            }
            let mut it = Vec::new();
            let mut s0 = Section::new("Passo 0");
            s0.line(format!("Começa no nó {}.", names[start]));
            it.push(s0);
            for (k, s) in steps.iter().enumerate() {
                let mut sec = Section::new(format!("Passo {}", k + 1));
                sec.line(format!("Conecta {} a {}: comprimento = {}", names[s.from], names[s.to], num(s.length)));
                sec.line(format!("Comprimento atualizado da árvore = {}", num(s.total)));
                if k + 1 == steps.len() {
                    sec.note("A última linha completa a árvore geradora mínima.");
                }
                it.push(sec);
            }
            (vec![h, t], it)
        }
        NetKind::Shortest => match algo {
            ShortAlgo::Dijkstra => {
                let (dist, pred, iters) = dijkstra(&d, start);
                let mut h = Section::new("Caminho mais curto: algoritmo de Dijkstra").sub(title);
                h.line(format!("A partir do nó {}", names[start]));
                let mut t = Section::new("Rotas mais curtas").headers(&["Para", "Distância", "Rota"]);
                for j in 0..n {
                    if j == start {
                        continue;
                    }
                    let r = route(&pred, start, j);
                    let rt = r
                        .as_ref()
                        .map(|r| r.iter().map(|&k| names[k].clone()).collect::<Vec<_>>().join(" → "))
                        .unwrap_or_else(|| "sem rota".into());
                    let st = if j == target { CellStyle::Green } else { CellStyle::Normal };
                    t.row(vec![cs(names[j].clone(), CellStyle::Head), cs(num(dist[j]), st), cs(rt, st)]);
                }
                if let Some(r) = route(&pred, start, target) {
                    for w in r.windows(2) {
                        highlight.push((w[0], w[1], num(d[w[0]][w[1]])));
                    }
                    h.line(format!("Destino {}: distância = {}", names[target], num(dist[target])));
                }
                let mut it = Vec::new();
                for (k, s) in iters.iter().enumerate() {
                    let mut sec = Section::new(format!("Iteração {}", k + 1)).headers(&["Nó", "Rótulo [distância, anterior]", "Situação"]);
                    sec.subtitle = format!("Nó tornado permanente: {}", names[s.permanent]);
                    for j in 0..n {
                        let lab = match s.labels[j] {
                            Some((dd, _)) if j == start => format!("[{}, –]", num(dd)),
                            Some((dd, p)) => format!("[{}, {}]", num(dd), names[p]),
                            None => "–".into(),
                        };
                        let st = if j == s.permanent { CellStyle::Green } else { CellStyle::Normal };
                        sec.row(vec![
                            cs(names[j].clone(), CellStyle::Head),
                            cs(lab, st),
                            c(if s.is_permanent[j] {
                                "permanente"
                            } else if s.labels[j].is_some() {
                                "temporário"
                            } else {
                                "–"
                            }),
                        ]);
                    }
                    it.push(sec);
                }
                (vec![h, t], it)
            }
            ShortAlgo::Floyd => {
                let iters = floyd(&d);
                let last = iters.last().unwrap();
                let mut h = Section::new("Caminho mais curto: algoritmo de Floyd").sub(title);
                h.line("Rotas mais curtas entre todos os pares de nós.".to_string());
                let mut t = Section::new("Rotas mais curtas").headers(&["De", "Para", "Distância", "Rota"]);
                for i in 0..n {
                    for j in 0..n {
                        if i == j {
                            continue;
                        }
                        let dist = last.d[i][j];
                        let rt = if dist.is_finite() {
                            floyd_route(&last.s, i, j).iter().map(|&k| names[k].clone()).collect::<Vec<_>>().join(" → ")
                        } else {
                            "sem rota".into()
                        };
                        let st = if i == start && j == target { CellStyle::Green } else { CellStyle::Normal };
                        t.row(vec![cs(names[i].clone(), CellStyle::Head), c(names[j].clone()), cs(num(dist), st), cs(rt, st)]);
                    }
                }
                if last.d[start][target].is_finite() {
                    let r = floyd_route(&last.s, start, target);
                    for w in r.windows(2) {
                        highlight.push((w[0], w[1], num(d[w[0]][w[1]])));
                    }
                }
                let mut it = Vec::new();
                for f in &iters {
                    let mut hd = vec![String::new()];
                    hd.extend(names.iter().cloned());
                    let mut sd = Section::new(if f.k == 0 {
                        "Iteração 0 (matrizes iniciais)".to_string()
                    } else {
                        format!("Iteração {} (nó pivô {})", f.k, names[f.k - 1])
                    })
                    .headers_owned(hd.clone());
                    sd.subtitle = "Matriz D (distâncias)".into();
                    for i in 0..n {
                        let mut r = vec![cs(names[i].clone(), CellStyle::Head)];
                        for j in 0..n {
                            let st = if f.k > 0 && (i == f.k - 1 || j == f.k - 1) { CellStyle::Pivot } else { CellStyle::Normal };
                            r.push(cs(if i == j { "–".into() } else { num(f.d[i][j]) }, st));
                        }
                        sd.row(r);
                    }
                    let mut ss = Section::new("").headers_owned(hd);
                    ss.subtitle = "Matriz S (nó intermediário)".into();
                    for i in 0..n {
                        let mut r = vec![cs(names[i].clone(), CellStyle::Head)];
                        for j in 0..n {
                            r.push(c(if i == j { "–".into() } else { names[f.s[i][j]].clone() }));
                        }
                        ss.row(r);
                    }
                    sd.notes = vec![];
                    it.push(sd);
                    it.push(ss);
                }
                (vec![h, t], it)
            }
        },
        NetKind::MaxFlow => {
            if start == target {
                return Err("Origem e destino devem ser nós diferentes.".into());
            }
            let caps: Vec<Vec<f64>> = d.iter().map(|r| r.iter().map(|v| if v.is_finite() { *v } else { 0.0 }).collect()).collect();
            let res = max_flow(&caps, start, target);
            let mut h = Section::new("Fluxo máximo: resumo").sub(title);
            h.line(format!("Fluxo máximo na rede = {}  (solução em {} iterações)", num(res.total), res.iters.len()));
            h.line(format!("Origem {} → destino {}", names[start], names[target]));
            let mut t = Section::new("Fluxos nos arcos").headers(&["De", "Para", "Capacidade", "Fluxo", "Folga"]);
            for i in 0..n {
                for j in 0..n {
                    if res.flows[i][j] > 1e-9 {
                        let cap = d[i][j];
                        t.row(vec![
                            cs(names[i].clone(), CellStyle::Head),
                            c(names[j].clone()),
                            c(num(cap)),
                            cs(num(res.flows[i][j]), CellStyle::Bold),
                            c(num(cap - res.flows[i][j])),
                        ]);
                        highlight.push((i, j, num(res.flows[i][j])));
                    }
                }
            }
            let mut it = Vec::new();
            for (k, f) in res.iters.iter().enumerate() {
                let mut s = Section::new(format!("Iteração {}", k + 1));
                s.line(format!("Caminho de aumento: {}", f.path.iter().map(|&p| names[p].clone()).collect::<Vec<_>>().join(" → ")));
                s.line(format!("Fluxo de avanço = {}", num(f.flow)));
                s.line(format!("Fluxo acumulado = {}", num(f.total)));
                it.push(s);
            }
            let mut end = Section::new("Fim");
            end.note(format!("Não há mais caminhos de aumento. O algoritmo termina na iteração {}.", res.iters.len() + 1));
            it.push(end);
            (vec![h, t], it)
        }
    };
    Ok(NetOutcome {
        results,
        iterations,
        highlight,
        start,
        target,
    })
}

pub fn report(input: &NetInput, kind: NetKind, results: &[Section], iterations: &[Section]) -> Option<String> {
    if results.is_empty() {
        return None;
    }
    let mut all = results.to_vec();
    all.extend(iterations.iter().cloned());
    Some(sections_to_text(&format!("{}: {}", kind.report_title(), input.title), &all))
}
