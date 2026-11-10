use super::common::*;
use super::{iterations_view, motion, results_view, solve_bar, tab_bar, title_row, Module, Tab};
use crate::format::{num, parse_num};
use crate::network::{dijkstra, floyd, floyd_route, max_flow, minimal_spanning_tree, route, INF};
use eframe::egui::{self, Align2, Pos2, RichText, Stroke, StrokeKind, Vec2};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NetKind {
    Mst,
    Shortest,
    MaxFlow,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ShortAlgo {
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
    fn blank(n: usize) -> Self {
        let mut s = Self {
            title: String::new(),
            n,
            names: vec![],
            matrix: vec![],
        };
        s.fit();
        s
    }
    fn fit(&mut self) {
        resize_names(&mut self.names, self.n, "N");
        resize_grid(&mut self.matrix, self.n, self.n);
    }
}

pub struct NetworkModule {
    pub kind: NetKind,
    pub input: NetInput,
    tab: Tab,
    results: Vec<Section>,
    iterations: Vec<Section>,
    shown: usize,
    error: Option<String>,
    start: usize,
    target: usize,
    algo: ShortAlgo,
    highlight: Vec<(usize, usize, String)>,
}

impl NetworkModule {
    pub fn new(kind: NetKind) -> Self {
        Self {
            kind,
            input: NetInput::blank(5),
            tab: Tab::Data,
            results: vec![],
            iterations: vec![],
            shown: 1,
            error: None,
            start: 0,
            target: 4,
            algo: ShortAlgo::Dijkstra,
            highlight: vec![],
        }
    }

    fn matrix(&self) -> Vec<Vec<f64>> {
        let n = self.input.n;
        (0..n)
            .map(|i| {
                (0..n)
                    .map(|j| {
                        if i == j {
                            INF
                        } else {
                            match parse_num(&self.input.matrix[i][j]) {
                                Some(v) if v.is_finite() => v,
                                _ => INF,
                            }
                        }
                    })
                    .collect()
            })
            .collect()
    }

    fn solve(&mut self) {
        self.error = None;
        self.highlight.clear();
        let d = self.matrix();
        let names = self.input.names.clone();
        let n = self.input.n;
        self.start = self.start.min(n - 1);
        self.target = self.target.min(n - 1);
        let title = if self.input.title.is_empty() { "Sem título".to_string() } else { self.input.title.clone() };
        match self.kind {
            NetKind::Mst => match minimal_spanning_tree(&d, self.start) {
                Err(e) => self.error = Some(e),
                Ok(steps) => {
                    let total = steps.last().map(|s| s.total).unwrap_or(0.0);
                    let mut h = Section::new("Árvore geradora mínima: resumo").sub(title);
                    h.line(format!("Comprimento da árvore geradora mínima = {}", num(total)));
                    let mut t = Section::new("Arcos da árvore").headers(&["De", "Para", "Comprimento"]);
                    for s in &steps {
                        t.row(vec![cs(names[s.from].clone(), CellStyle::Head), c(names[s.to].clone()), c(num(s.length))]);
                        self.highlight.push((s.from, s.to, num(s.length)));
                    }
                    self.results = vec![h, t];
                    let mut it = Vec::new();
                    let mut s0 = Section::new("Passo 0");
                    s0.line(format!("Começa no nó {}.", names[self.start]));
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
                    self.iterations = it;
                }
            },
            NetKind::Shortest => match self.algo {
                ShortAlgo::Dijkstra => {
                    let (dist, pred, iters) = dijkstra(&d, self.start);
                    let mut h = Section::new("Caminho mais curto: algoritmo de Dijkstra").sub(title);
                    h.line(format!("A partir do nó {}", names[self.start]));
                    let mut t = Section::new("Rotas mais curtas").headers(&["Para", "Distância", "Rota"]);
                    for j in 0..n {
                        if j == self.start {
                            continue;
                        }
                        let r = route(&pred, self.start, j);
                        let rt = r.as_ref().map(|r| r.iter().map(|&k| names[k].clone()).collect::<Vec<_>>().join(" → ")).unwrap_or_else(|| "sem rota".into());
                        let st = if j == self.target { CellStyle::Green } else { CellStyle::Normal };
                        t.row(vec![cs(names[j].clone(), CellStyle::Head), cs(num(dist[j]), st), cs(rt, st)]);
                    }
                    if let Some(r) = route(&pred, self.start, self.target) {
                        for w in r.windows(2) {
                            self.highlight.push((w[0], w[1], num(d[w[0]][w[1]])));
                        }
                        h.line(format!("Destino {}: distância = {}", names[self.target], num(dist[self.target])));
                    }
                    self.results = vec![h, t];
                    let mut it = Vec::new();
                    for (k, s) in iters.iter().enumerate() {
                        let mut sec = Section::new(format!("Iteração {}", k + 1)).headers(&["Nó", "Rótulo [distância, anterior]", "Situação"]);
                        sec.subtitle = format!("Nó tornado permanente: {}", names[s.permanent]);
                        for j in 0..n {
                            let lab = match s.labels[j] {
                                Some((dd, _)) if j == self.start => format!("[{}, –]", num(dd)),
                                Some((dd, p)) => format!("[{}, {}]", num(dd), names[p]),
                                None => "–".into(),
                            };
                            let st = if j == s.permanent { CellStyle::Green } else { CellStyle::Normal };
                            sec.row(vec![
                                cs(names[j].clone(), CellStyle::Head),
                                cs(lab, st),
                                c(if s.is_permanent[j] { "permanente" } else if s.labels[j].is_some() { "temporário" } else { "–" }),
                            ]);
                        }
                        it.push(sec);
                    }
                    self.iterations = it;
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
                            let st = if i == self.start && j == self.target { CellStyle::Green } else { CellStyle::Normal };
                            t.row(vec![cs(names[i].clone(), CellStyle::Head), c(names[j].clone()), cs(num(dist), st), cs(rt, st)]);
                        }
                    }
                    if last.d[self.start][self.target].is_finite() {
                        let r = floyd_route(&last.s, self.start, self.target);
                        for w in r.windows(2) {
                            self.highlight.push((w[0], w[1], num(d[w[0]][w[1]])));
                        }
                    }
                    self.results = vec![h, t];
                    let mut it = Vec::new();
                    for f in &iters {
                        let mut hd = vec![String::new()];
                        hd.extend(names.iter().cloned());
                        let mut sd = Section::new(if f.k == 0 { "Iteração 0 (matrizes iniciais)".to_string() } else { format!("Iteração {} (nó pivô {})", f.k, names[f.k - 1]) })
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
                        let mut joined = sd;
                        joined.notes = vec![];
                        it.push(joined);
                        it.push(ss);
                    }
                    self.iterations = it;
                }
            },
            NetKind::MaxFlow => {
                if self.start == self.target {
                    self.error = Some("Origem e destino devem ser nós diferentes.".into());
                    return;
                }
                let res = max_flow(&d.iter().map(|r| r.iter().map(|v| if v.is_finite() { *v } else { 0.0 }).collect()).collect::<Vec<Vec<f64>>>(), self.start, self.target);
                let mut h = Section::new("Fluxo máximo: resumo").sub(title);
                h.line(format!("Fluxo máximo na rede = {}  (solução em {} iterações)", num(res.total), res.iters.len()));
                h.line(format!("Origem {} → destino {}", names[self.start], names[self.target]));
                let mut t = Section::new("Fluxos nos arcos").headers(&["De", "Para", "Capacidade", "Fluxo", "Folga"]);
                for i in 0..n {
                    for j in 0..n {
                        if res.flows[i][j] > 1e-9 {
                            let cap = d[i][j];
                            t.row(vec![cs(names[i].clone(), CellStyle::Head), c(names[j].clone()), c(num(cap)), cs(num(res.flows[i][j]), CellStyle::Bold), c(num(cap - res.flows[i][j]))]);
                            self.highlight.push((i, j, num(res.flows[i][j])));
                        }
                    }
                }
                self.results = vec![h, t];
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
                self.iterations = it;
            }
        }
        if self.error.is_none() {
            self.shown = 1;
            self.tab = Tab::Results;
        }
    }

    fn draw_network(&self, ui: &mut egui::Ui) {
        let d = self.matrix();
        let n = self.input.n;
        let h = (ui.available_height() - 10.0).max(360.0);
        let (resp, painter) = ui.allocate_painter(Vec2::new(ui.available_width(), h), egui::Sense::hover());
        let r = resp.rect;
        painter.rect_filled(r, 4.0, card_fill());
        painter.rect_stroke(r, 4.0, Stroke::new(1.0, border()), StrokeKind::Inside);
        let elapsed = motion::clock(ui.ctx(), ("net_graph", self.key()), 0.5 + self.highlight.len() as f32 * 0.22 + 0.4);
        let center = r.center();
        let rad = (r.width().min(r.height()) / 2.0 - 50.0).max(60.0);
        let pos: Vec<Pos2> = (0..n)
            .map(|i| {
                let a = std::f32::consts::TAU * i as f32 / n as f32 - std::f32::consts::FRAC_PI_2;
                center + Vec2::new(a.cos() * rad, a.sin() * rad)
            })
            .collect();
        let directed = self.kind != NetKind::Mst;
        for i in 0..n {
            for j in 0..n {
                if i == j || !d[i][j].is_finite() {
                    continue;
                }
                if !directed && j < i && d[j][i].is_finite() {
                    continue;
                }
                let found = self.highlight.iter().position(|(a, b, _)| (*a == i && *b == j) || (!directed && *a == j && *b == i));
                let lit = found.map(|k| motion::ease_in_out_cubic(motion::phase(elapsed, 0.45 + k as f32 * 0.22, 0.3))).unwrap_or(0.0);
                let done = lit >= 1.0;
                let reversed = found.is_some_and(|k| self.highlight[k].0 == j);
                let a = pos[i];
                let b = pos[j];
                let dir = (b - a).normalized();
                let bidir = directed && d[j][i].is_finite();
                let off = if bidir { Vec2::new(-dir.y, dir.x) * 5.0 } else { Vec2::ZERO };
                let a2 = a + dir * 20.0 + off;
                let b2 = b - dir * 22.0 + off;
                painter.line_segment([a2, b2], Stroke::new(1.3, edge_line()));
                if reversed {
                    partial_segment(&painter, b2, a2, lit, Stroke::new(3.0, red()));
                } else {
                    partial_segment(&painter, a2, b2, lit, Stroke::new(3.0, red()));
                }
                if directed {
                    let (col, w) = if done { (red(), 3.0) } else { (edge_line(), 1.3) };
                    let normal = Vec2::new(-dir.y, dir.x);
                    let size = 6.0 + w;
                    let base = b2 - dir * size * 1.6;
                    painter.add(egui::Shape::convex_polygon(
                        vec![b2, base + normal * size * 0.6, base - normal * size * 0.6],
                        col,
                        Stroke::NONE,
                    ));
                }
                let label = match found {
                    Some(k) if done && self.kind == NetKind::MaxFlow => format!("{}/{}", self.highlight[k].2, num(d[i][j])),
                    _ => num(d[i][j]),
                };
                let text_color = if done { highlight_text() } else { text_soft() };
                let font = if done { mono_bold(12.0) } else { mono(12.0) };
                let galley = painter.layout_no_wrap(label, font, text_color);
                let at = a2.lerp(b2, 0.5) + off * 2.0;
                let rect = egui::Rect::from_center_size(at, galley.size() + Vec2::new(6.0, 2.0));
                painter.rect_filled(rect, 2.0, label_backing());
                painter.galley(rect.min + Vec2::new(3.0, 1.0), galley, text_color);
            }
        }
        for i in 0..n {
            let special = (self.kind != NetKind::Mst && (i == self.start || i == self.target)) || (self.kind == NetKind::Mst && i == self.start);
            let fill = if special { accent() } else { node_fill() };
            let pop = motion::phase(elapsed, i as f32 * 0.04, 0.34);
            painter.circle_filled(pos[i], 18.0 * motion::ease_out_back(pop), fill);
            if pop > 0.5 {
                painter.text(pos[i], Align2::CENTER_CENTER, &self.input.names[i], semibold(12.5), on_color(fill));
            }
        }
    }

    fn node_combo(ui: &mut egui::Ui, id: &str, label: &str, value: &mut usize, names: &[String]) {
        ui.label(RichText::new(label).color(text_muted()));
        egui::ComboBox::from_id_salt(id)
            .selected_text(names.get(*value).cloned().unwrap_or_default())
            .show_ui(ui, |ui| {
                for (i, nm) in names.iter().enumerate() {
                    ui.selectable_value(value, i, nm);
                }
            });
    }
}

impl Module for NetworkModule {
    fn solve_now(&mut self) {
        self.solve();
    }

    fn set_tab(&mut self, t: Tab) {
        self.tab = t;
    }

    fn key(&self) -> &'static str {
        match self.kind {
            NetKind::Mst => "minimal_spanning_tree",
            NetKind::Shortest => "shortest_route",
            NetKind::MaxFlow => "maximal_flow",
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        tab_bar(ui, &mut self.tab, &[(Tab::Data, "Dados"), (Tab::Results, "Resultado"), (Tab::Iterations, "Iterações"), (Tab::Graph, "Rede")]);
        match self.tab {
            Tab::Data => {
                card(ui, |ui| {
                    title_row(ui, &mut self.input.title);
                    ui.add_space(6.0);
                    if stepper(ui, "Nº de nós", &mut self.input.n, 2, crate::mechanics::MAX_PROBLEM_SIZE) {
                        self.input.fit();
                    }
                    let names = self.input.names.clone();
                    self.start = self.start.min(self.input.n - 1);
                    self.target = self.target.min(self.input.n - 1);
                    ui.horizontal(|ui| match self.kind {
                        NetKind::Mst => Self::node_combo(ui, "mst_start", "Nó inicial:", &mut self.start, &names),
                        NetKind::Shortest => {
                            ui.radio_value(&mut self.algo, ShortAlgo::Dijkstra, "Dijkstra");
                            ui.radio_value(&mut self.algo, ShortAlgo::Floyd, "Floyd (todas as rotas)");
                            ui.add_space(16.0);
                            Self::node_combo(ui, "sr_from", "De:", &mut self.start, &names);
                            Self::node_combo(ui, "sr_to", "Para:", &mut self.target, &names);
                        }
                        NetKind::MaxFlow => {
                            Self::node_combo(ui, "mf_from", "Origem:", &mut self.start, &names);
                            Self::node_combo(ui, "mf_to", "Destino:", &mut self.target, &names);
                        }
                    });
                    let hint = match self.kind {
                        NetKind::Mst => "Informe a distância entre os nós (basta preencher uma das células i→j ou j→i). Vazio = sem ligação.",
                        NetKind::Shortest => "Linha = nó de origem, coluna = nó de destino. Vazio = sem arco.",
                        NetKind::MaxFlow => "Capacidade do arco da linha (de) para a coluna (para). Vazio = sem arco.",
                    };
                    ui.label(RichText::new(hint).small().color(text_muted()));
                });
                ui.add_space(10.0);
                let n = self.input.n;
                card(ui, |ui| {
                    egui::ScrollArea::both().id_salt("net_grid").max_height(540.0).show(ui, |ui| {
                        egui::Grid::new("net_in").spacing([6.0, 6.0]).show(ui, |ui| {
                            head(ui, "De \\ Para");
                            for j in 0..n {
                                head(ui, &self.input.names[j]);
                            }
                            ui.end_row();
                            for i in 0..n {
                                ui.add(egui::TextEdit::singleline(&mut self.input.names[i]).desired_width(70.0).text_color(accent()));
                                for j in 0..n {
                                    if i == j {
                                        ui.label(RichText::new("–").color(text_muted()));
                                    } else {
                                        cell_edit(ui, &mut self.input.matrix[i][j], 56.0);
                                    }
                                }
                                ui.end_row();
                            }
                        });
                    });
                });
                if let Some(e) = &self.error {
                    ui.add_space(8.0);
                    error_box(ui, e);
                }
                if solve_bar(ui) {
                    self.solve();
                }
            }
            Tab::Results => results_view(ui, "net_res", &self.results),
            Tab::Iterations => iterations_view(ui, "net_it", &self.iterations, &mut self.shown),
            Tab::Graph => {
                if self.highlight.is_empty() {
                    ui.label(RichText::new("Rede informada (resolva para destacar a solução em vermelho).").color(text_muted()));
                } else {
                    ui.label(RichText::new("Arcos da solução em vermelho.").color(text_muted()));
                }
                self.draw_network(ui);
            }
        }
    }

    fn report(&self) -> Option<String> {
        if self.results.is_empty() {
            return None;
        }
        let mut all = self.results.clone();
        all.extend(self.iterations.iter().cloned());
        let t = match self.kind {
            NetKind::Mst => "ÁRVORE GERADORA MÍNIMA",
            NetKind::Shortest => "CAMINHO MAIS CURTO",
            NetKind::MaxFlow => "FLUXO MÁXIMO",
        };
        Some(sections_to_text(&format!("{}: {}", t, self.input.title), &all))
    }

    fn save_json(&self) -> serde_json::Value {
        serde_json::to_value(&self.input).unwrap_or_default()
    }

    fn load_json(&mut self, v: serde_json::Value) -> Result<(), String> {
        let mut inp: NetInput = serde_json::from_value(v).map_err(|e| e.to_string())?;
        inp.fit();
        self.input = inp;
        self.results.clear();
        self.iterations.clear();
        self.highlight.clear();
        self.tab = Tab::Data;
        Ok(())
    }

    fn new_problem(&mut self) {
        self.input = NetInput::blank(5);
        self.results.clear();
        self.iterations.clear();
        self.highlight.clear();
        self.error = None;
        self.tab = Tab::Data;
    }

    fn load_example(&mut self) {
        let mut inp = NetInput::blank(6);
        let arcs: &[(usize, usize, &str)] = match self.kind {
            NetKind::Mst => &[(0, 1, "4"), (0, 2, "3"), (1, 2, "2"), (1, 3, "6"), (2, 3, "5"), (2, 4, "7"), (3, 4, "3"), (3, 5, "4"), (4, 5, "2")],
            NetKind::Shortest => &[(0, 1, "7"), (0, 2, "3"), (1, 3, "2"), (2, 1, "2"), (2, 3, "8"), (2, 4, "6"), (3, 5, "4"), (4, 3, "1"), (4, 5, "7")],
            NetKind::MaxFlow => &[(0, 1, "12"), (0, 2, "9"), (1, 2, "4"), (1, 3, "8"), (2, 4, "10"), (3, 4, "3"), (3, 5, "9"), (4, 5, "11")],
        };
        for &(i, j, v) in arcs {
            inp.matrix[i][j] = v.to_string();
        }
        inp.title = "Rede de exemplo com 6 nós".into();
        self.input = inp;
        self.start = 0;
        self.target = 5;
        self.results.clear();
        self.iterations.clear();
        self.highlight.clear();
        self.tab = Tab::Data;
    }
}
