use serde::{Deserialize, Serialize};

pub const EPS: f64 = crate::mechanics::EPSILON;
pub const MAX_ITERATIONS: usize = 2000;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Sense {
    Max,
    Min,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Rel {
    Le,
    Ge,
    Eq,
}

impl Rel {
    pub fn symbol(&self) -> &'static str {
        match self {
            Rel::Le => "<=",
            Rel::Ge => ">=",
            Rel::Eq => "=",
        }
    }
    pub fn parse(s: &str) -> Rel {
        let t = s.trim();
        if t.starts_with('>') {
            Rel::Ge
        } else if t.starts_with('=') {
            Rel::Eq
        } else {
            Rel::Le
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Method {
    AllSlack,
    MMethod,
    TwoPhase,
    DualSimplex,
}

impl Method {
    pub fn label(&self) -> &'static str {
        match self {
            Method::AllSlack => "Folgas (todas <=)",
            Method::MMethod => "Método do M grande",
            Method::TwoPhase => "Duas fases",
            Method::DualSimplex => "Dual simplex",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ColKind {
    Decision,
    Slack,
    Surplus,
    Artificial,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Status {
    Optimal,
    Unbounded,
    Infeasible,
}

#[derive(Clone, Debug)]
pub struct StdForm {
    pub sense: Sense,
    pub names: Vec<String>,
    pub c: Vec<f64>,
    pub a: Vec<Vec<f64>>,
    pub rel: Vec<Rel>,
    pub b: Vec<f64>,
}

#[derive(Clone, Debug)]
pub struct Tableau {
    pub phase: u8,
    pub iteration: usize,
    pub col_names: Vec<String>,
    pub col_kind: Vec<ColKind>,
    pub blocked: Vec<bool>,
    pub cost: Vec<f64>,
    pub sense: Sense,
    pub rows: Vec<Vec<f64>>,
    pub rhs: Vec<f64>,
    pub basis: Vec<usize>,
    pub zrow: Vec<f64>,
    pub zval: f64,
    pub entering: Option<usize>,
    pub leaving: Option<usize>,
    pub note: String,
}

impl Tableau {
    pub fn recompute_z(&mut self) {
        let nc = self.col_names.len();
        let mut z = vec![0.0; nc];
        let mut zv = 0.0;
        for (i, &bi) in self.basis.iter().enumerate() {
            let cb = self.cost[bi];
            if cb != 0.0 {
                for j in 0..nc {
                    z[j] += cb * self.rows[i][j];
                }
                zv += cb * self.rhs[i];
            }
        }
        for j in 0..nc {
            z[j] -= self.cost[j];
            if z[j].abs() < EPS {
                z[j] = 0.0;
            }
        }
        self.zrow = z;
        self.zval = zv;
    }

    pub fn is_basic(&self, j: usize) -> bool {
        self.basis.contains(&j)
    }

    fn pivot(&mut self, r: usize, e: usize) {
        let p = self.rows[r][e];
        let nc = self.col_names.len();
        for j in 0..nc {
            self.rows[r][j] /= p;
        }
        self.rhs[r] /= p;
        for i in 0..self.rows.len() {
            if i != r {
                let f = self.rows[i][e];
                if f != 0.0 {
                    for j in 0..nc {
                        self.rows[i][j] -= f * self.rows[r][j];
                        if self.rows[i][j].abs() < 1e-12 {
                            self.rows[i][j] = 0.0;
                        }
                    }
                    self.rhs[i] -= f * self.rhs[r];
                    if self.rhs[i].abs() < 1e-11 {
                        self.rhs[i] = 0.0;
                    }
                }
            }
        }
        self.basis[r] = e;
        self.recompute_z();
    }

    fn entering_primal(&self) -> Option<usize> {
        let mut best: Option<usize> = None;
        let mut best_val = 0.0;
        for j in 0..self.col_names.len() {
            if self.blocked[j] || self.is_basic(j) {
                continue;
            }
            let v = match self.sense {
                Sense::Max => -self.zrow[j],
                Sense::Min => self.zrow[j],
            };
            if v > EPS && v > best_val + EPS {
                best_val = v;
                best = Some(j);
            }
        }
        best
    }

    fn leaving_primal(&self, e: usize) -> Option<usize> {
        let mut best: Option<usize> = None;
        let mut best_ratio = f64::INFINITY;
        for i in 0..self.rows.len() {
            let a = self.rows[i][e];
            if a > EPS {
                let ratio = self.rhs[i] / a;
                if ratio < best_ratio - 1e-12 {
                    best_ratio = ratio;
                    best = Some(i);
                }
            }
        }
        best
    }

    fn leaving_dual(&self) -> Option<usize> {
        let mut best: Option<usize> = None;
        let mut best_val = -EPS;
        for i in 0..self.rows.len() {
            if self.rhs[i] < best_val - 1e-12 {
                best_val = self.rhs[i];
                best = Some(i);
            }
        }
        best
    }

    fn entering_dual(&self, r: usize) -> Option<usize> {
        let mut best: Option<usize> = None;
        let mut best_ratio = f64::INFINITY;
        for j in 0..self.col_names.len() {
            if self.blocked[j] || self.is_basic(j) {
                continue;
            }
            let a = self.rows[r][j];
            if a < -EPS {
                let ratio = (self.zrow[j] / a).abs();
                if ratio < best_ratio - 1e-12 {
                    best_ratio = ratio;
                    best = Some(j);
                }
            }
        }
        best
    }

    pub fn is_optimal_for_dual(&self) -> bool {
        for j in 0..self.col_names.len() {
            if self.blocked[j] || self.is_basic(j) {
                continue;
            }
            match self.sense {
                Sense::Max => {
                    if self.zrow[j] < -EPS {
                        return false;
                    }
                }
                Sense::Min => {
                    if self.zrow[j] > EPS {
                        return false;
                    }
                }
            }
        }
        true
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Stage {
    PhaseOne,
    Main,
    Done,
}

#[derive(Clone, Debug)]
pub struct SimplexRun {
    pub method: Method,
    pub history: Vec<Tableau>,
    pub status: Option<Status>,
    pub alternative: bool,
    pub orig_cost: Vec<f64>,
    pub orig_sense: Sense,
    pub row_sign: Vec<f64>,
    pub init_basis_col: Vec<usize>,
    pub n_dec: usize,
    pub big_m: f64,
    pub message: String,
    stage: Stage,
}

pub fn all_le_after_normalising(form: &StdForm) -> bool {
    form.rel.iter().zip(form.b.iter()).all(|(r, &b)| match r {
        Rel::Le => b >= 0.0,
        Rel::Ge => b < 0.0,
        Rel::Eq => false,
    })
}

pub fn has_equations(form: &StdForm) -> bool {
    form.rel.iter().any(|r| *r == Rel::Eq)
}

impl SimplexRun {
    pub fn new(form: &StdForm, method: Method, big_m: f64) -> Result<SimplexRun, String> {
        let m = form.a.len();
        let n = form.c.len();
        let mut rel = form.rel.clone();
        let mut a = form.a.clone();
        let mut b = form.b.clone();
        let mut sign = vec![1.0; m];
        let dual = method == Method::DualSimplex;
        for i in 0..m {
            if dual {
                if rel[i] == Rel::Eq {
                    return Err("Equações devem ser convertidas em desigualdades antes de usar o Dual Simplex.".into());
                }
                if rel[i] == Rel::Ge {
                    sign[i] = -1.0;
                    rel[i] = Rel::Le;
                }
            } else if b[i] < 0.0 {
                sign[i] = -1.0;
                rel[i] = match rel[i] {
                    Rel::Le => Rel::Ge,
                    Rel::Ge => Rel::Le,
                    Rel::Eq => Rel::Eq,
                };
            }
            if sign[i] < 0.0 {
                for v in a[i].iter_mut() {
                    *v = -*v;
                }
                b[i] = -b[i];
            }
        }
        if method == Method::AllSlack && rel.iter().any(|r| *r != Rel::Le) {
            return Err("O método das folgas exige que todas as restrições sejam <= com lado direito não negativo.".into());
        }
        let mut col_names: Vec<String> = form.names.clone();
        let mut col_kind = vec![ColKind::Decision; n];
        let mut cols: Vec<Vec<f64>> = (0..n).map(|j| (0..m).map(|i| a[i][j]).collect()).collect();
        let mut init_basis_col = vec![usize::MAX; m];
        for i in 0..m {
            match rel[i] {
                Rel::Le => {
                    let mut c = vec![0.0; m];
                    c[i] = 1.0;
                    cols.push(c);
                    col_names.push(format!("s{}", i + 1));
                    col_kind.push(ColKind::Slack);
                    init_basis_col[i] = cols.len() - 1;
                }
                Rel::Ge => {
                    let mut c = vec![0.0; m];
                    c[i] = -1.0;
                    cols.push(c);
                    col_names.push(format!("s{}", i + 1));
                    col_kind.push(ColKind::Surplus);
                }
                Rel::Eq => {}
            }
        }
        for i in 0..m {
            if rel[i] != Rel::Le {
                let mut c = vec![0.0; m];
                c[i] = 1.0;
                cols.push(c);
                col_names.push(format!("R{}", i + 1));
                col_kind.push(ColKind::Artificial);
                init_basis_col[i] = cols.len() - 1;
            }
        }
        let nc = cols.len();
        let rows: Vec<Vec<f64>> = (0..m).map(|i| (0..nc).map(|j| cols[j][i]).collect()).collect();
        let mut orig_cost = vec![0.0; nc];
        orig_cost[..n].copy_from_slice(&form.c);
        let has_art = col_kind.iter().any(|k| *k == ColKind::Artificial);
        let (cost, sense, phase, stage) = match method {
            Method::TwoPhase if has_art => {
                let mut c = vec![0.0; nc];
                for j in 0..nc {
                    if col_kind[j] == ColKind::Artificial {
                        c[j] = 1.0;
                    }
                }
                (c, Sense::Min, 1u8, Stage::PhaseOne)
            }
            Method::MMethod if has_art => {
                let mut c = orig_cost.clone();
                for j in 0..nc {
                    if col_kind[j] == ColKind::Artificial {
                        c[j] = match form.sense {
                            Sense::Max => -big_m,
                            Sense::Min => big_m,
                        };
                    }
                }
                (c, form.sense, 0u8, Stage::Main)
            }
            _ => (orig_cost.clone(), form.sense, 0u8, Stage::Main),
        };
        let mut t = Tableau {
            phase,
            iteration: 1,
            col_names,
            col_kind,
            blocked: vec![false; nc],
            cost,
            sense,
            rows,
            rhs: b,
            basis: init_basis_col.clone(),
            zrow: vec![],
            zval: 0.0,
            entering: None,
            leaving: None,
            note: String::new(),
        };
        t.recompute_z();
        if dual && !t.is_optimal_for_dual() {
            return Err("Dual simplex não é aplicável diretamente: a linha z inicial não é ótima.".into());
        }
        Ok(SimplexRun {
            method,
            history: vec![t],
            status: None,
            alternative: false,
            orig_cost,
            orig_sense: form.sense,
            row_sign: sign,
            init_basis_col,
            n_dec: n,
            big_m,
            message: String::new(),
            stage,
        })
    }

    pub fn current(&self) -> &Tableau {
        self.history.last().unwrap()
    }

    pub fn finished(&self) -> bool {
        self.stage == Stage::Done
    }

    pub fn step(&mut self) -> bool {
        if self.stage == Stage::Done {
            return false;
        }
        if self.history.len() > MAX_ITERATIONS {
            self.finish(Status::Infeasible, "Limite de iterações atingido.".into());
            return false;
        }
        let cur = self.current().clone();
        if self.method == Method::DualSimplex {
            return self.dual_step(cur);
        }
        match cur.entering_primal() {
            None => {
                if self.stage == Stage::PhaseOne {
                    if cur.zval > 1e-7 {
                        self.finish(Status::Infeasible, "A Fase 1 mostra que o problema não tem solução viável.".into());
                        return false;
                    }
                    self.start_phase_two(cur);
                    return true;
                }
                let infeasible = cur.basis.iter().enumerate().any(|(i, &bj)| {
                    cur.col_kind[bj] == ColKind::Artificial && cur.rhs[i] > 1e-7
                });
                if infeasible {
                    self.finish(Status::Infeasible, "Não há solução viável: variável artificial positiva no último quadro.".into());
                    return false;
                }
                self.alternative = (0..cur.col_names.len()).any(|j| {
                    !cur.blocked[j]
                        && !cur.is_basic(j)
                        && cur.col_kind[j] != ColKind::Artificial
                        && cur.zrow[j].abs() < 1e-9
                });
                let msg = if self.alternative {
                    "Quadro ótimo. Há soluções ótimas alternativas.".to_string()
                } else {
                    "Quadro ótimo.".to_string()
                };
                self.finish(Status::Optimal, msg);
                false
            }
            Some(e) => match cur.leaving_primal(e) {
                None => {
                    if let Some(last) = self.history.last_mut() {
                        last.entering = Some(e);
                    }
                    self.finish(Status::Unbounded, "O último quadro mostra que a solução é ilimitada.".into());
                    false
                }
                Some(r) => {
                    {
                        let last = self.history.last_mut().unwrap();
                        last.entering = Some(e);
                        last.leaving = Some(r);
                    }
                    let mut next = cur;
                    next.entering = None;
                    next.leaving = None;
                    next.note.clear();
                    next.pivot(r, e);
                    next.iteration += 1;
                    self.history.push(next);
                    true
                }
            },
        }
    }

    fn dual_step(&mut self, cur: Tableau) -> bool {
        match cur.leaving_dual() {
            None => {
                self.alternative = (0..cur.col_names.len()).any(|j| {
                    !cur.blocked[j] && !cur.is_basic(j) && cur.zrow[j].abs() < 1e-9
                });
                self.finish(Status::Optimal, "Quadro ótimo (viável).".into());
                false
            }
            Some(r) => match cur.entering_dual(r) {
                None => {
                    if let Some(last) = self.history.last_mut() {
                        last.leaving = Some(r);
                    }
                    self.finish(Status::Infeasible, "Não há solução viável (linha em vermelho no último quadro).".into());
                    false
                }
                Some(e) => {
                    {
                        let last = self.history.last_mut().unwrap();
                        last.entering = Some(e);
                        last.leaving = Some(r);
                    }
                    let mut next = cur;
                    next.entering = None;
                    next.leaving = None;
                    next.pivot(r, e);
                    next.iteration += 1;
                    self.history.push(next);
                    true
                }
            },
        }
    }

    fn start_phase_two(&mut self, cur: Tableau) {
        let mut next = cur;
        next.entering = None;
        next.leaving = None;
        for i in 0..next.rows.len() {
            let bj = next.basis[i];
            if next.col_kind[bj] == ColKind::Artificial {
                let pick = (0..next.col_names.len()).find(|&j| {
                    next.col_kind[j] != ColKind::Artificial
                        && !next.is_basic(j)
                        && next.rows[i][j].abs() > 1e-7
                });
                if let Some(j) = pick {
                    next.pivot(i, j);
                }
            }
        }
        for j in 0..next.col_names.len() {
            if next.col_kind[j] == ColKind::Artificial {
                next.blocked[j] = true;
            }
        }
        next.cost = self.orig_cost.clone();
        next.sense = self.orig_sense;
        next.phase = 2;
        next.iteration += 1;
        next.recompute_z();
        next.note = "Fim da Fase 1: solução viável encontrada. Começa a Fase 2.".into();
        self.history.push(next);
        self.stage = Stage::Main;
    }

    fn finish(&mut self, status: Status, message: String) {
        self.status = Some(status);
        self.message = message.clone();
        if let Some(last) = self.history.last_mut() {
            if last.note.is_empty() {
                last.note = message;
            } else {
                last.note = format!("{} {}", last.note, message);
            }
        }
        self.stage = Stage::Done;
    }

    pub fn run_all(&mut self) {
        while self.step() {}
    }

    pub fn column_values(&self) -> Vec<f64> {
        let t = self.current();
        let mut v = vec![0.0; t.col_names.len()];
        for (i, &bj) in t.basis.iter().enumerate() {
            v[bj] = t.rhs[i];
        }
        v
    }

    pub fn binv(&self) -> Vec<Vec<f64>> {
        let t = self.current();
        let m = t.rows.len();
        (0..m)
            .map(|k| (0..m).map(|i| t.rows[k][self.init_basis_col[i]]).collect())
            .collect()
    }

    pub fn duals_std(&self) -> Vec<f64> {
        let t = self.current();
        let binv = self.binv();
        let m = t.rows.len();
        (0..m)
            .map(|i| {
                (0..m)
                    .map(|k| self.orig_cost[t.basis[k]] * binv[k][i])
                    .sum::<f64>()
            })
            .collect()
    }

    pub fn reduced_orig(&self) -> Vec<f64> {
        let t = self.current();
        let nc = t.col_names.len();
        (0..nc)
            .map(|j| {
                let s: f64 = t
                    .basis
                    .iter()
                    .enumerate()
                    .map(|(k, &bk)| self.orig_cost[bk] * t.rows[k][j])
                    .sum();
                let v = s - self.orig_cost[j];
                if v.abs() < 1e-9 {
                    0.0
                } else {
                    v
                }
            })
            .collect()
    }

    pub fn objective_orig(&self) -> f64 {
        let vals = self.column_values();
        vals.iter().zip(self.orig_cost.iter()).map(|(a, b)| a * b).sum()
    }

    pub fn cost_range(&self, col: usize) -> (f64, f64) {
        let t = self.current();
        let red = self.reduced_orig();
        let c = self.orig_cost[col];
        let max = self.orig_sense == Sense::Max;
        if let Some(r) = t.basis.iter().position(|&b| b == col) {
            let mut lo = f64::NEG_INFINITY;
            let mut hi = f64::INFINITY;
            for k in 0..t.col_names.len() {
                if k == col || t.blocked[k] || t.is_basic(k) || t.col_kind[k] == ColKind::Artificial {
                    continue;
                }
                let a = t.rows[r][k];
                if a.abs() < 1e-12 {
                    continue;
                }
                let bound = -red[k] / a;
                let lower_side = (a > 0.0) == max;
                if lower_side {
                    lo = lo.max(bound);
                } else {
                    hi = hi.min(bound);
                }
            }
            (c + lo, c + hi)
        } else if max {
            (f64::NEG_INFINITY, c + red[col])
        } else {
            (c + red[col], f64::INFINITY)
        }
    }

    pub fn rhs_range_std(&self, row: usize) -> (f64, f64) {
        let t = self.current();
        let binv = self.binv();
        let mut lo = f64::NEG_INFINITY;
        let mut hi = f64::INFINITY;
        for k in 0..t.rows.len() {
            let d = binv[k][row];
            if d.abs() < 1e-12 {
                continue;
            }
            let bound = -t.rhs[k] / d;
            if d > 0.0 {
                lo = lo.max(bound);
            } else {
                hi = hi.min(bound);
            }
        }
        if self.row_sign[row] < 0.0 {
            (-hi, -lo)
        } else {
            (lo, hi)
        }
    }
}
