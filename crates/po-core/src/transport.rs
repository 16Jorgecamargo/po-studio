use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum StartMethod {
    NorthWest,
    LeastCost,
    Vogel,
}

impl StartMethod {
    pub fn label(&self) -> &'static str {
        match self {
            StartMethod::NorthWest => "Canto Noroeste",
            StartMethod::LeastCost => "Custo Mínimo",
            StartMethod::Vogel => "Vogel",
        }
    }
}

#[derive(Clone, Debug)]
pub struct TransportProblem {
    pub sources: Vec<String>,
    pub dests: Vec<String>,
    pub cost: Vec<Vec<f64>>,
    pub supply: Vec<f64>,
    pub demand: Vec<f64>,
}

#[derive(Clone, Debug)]
pub struct TransportTableau {
    pub alloc: Vec<Vec<f64>>,
    pub basic: Vec<Vec<bool>>,
    pub u: Vec<f64>,
    pub v: Vec<f64>,
    pub reduced: Vec<Vec<f64>>,
    pub entering: Option<(usize, usize)>,
    pub loop_cells: Vec<(usize, usize)>,
    pub theta: f64,
    pub leaving: Option<(usize, usize)>,
    pub cost: f64,
}

#[derive(Clone, Debug)]
pub struct TransportSolution {
    pub problem: TransportProblem,
    pub method: StartMethod,
    pub tableaus: Vec<TransportTableau>,
    pub optimal: bool,
}

pub fn balance(p: &TransportProblem) -> TransportProblem {
    let mut q = p.clone();
    let ts: f64 = p.supply.iter().sum();
    let td: f64 = p.demand.iter().sum();
    if ts > td + 1e-9 {
        q.dests.push("DummyD".into());
        q.demand.push(ts - td);
        for row in q.cost.iter_mut() {
            row.push(0.0);
        }
    } else if td > ts + 1e-9 {
        q.sources.push("DummyS".into());
        q.supply.push(td - ts);
        q.cost.push(vec![0.0; q.dests.len()]);
    }
    q
}

fn initial(p: &TransportProblem, method: StartMethod) -> (Vec<Vec<f64>>, Vec<Vec<bool>>) {
    let m = p.supply.len();
    let n = p.demand.len();
    let mut s = p.supply.clone();
    let mut d = p.demand.clone();
    let mut alloc = vec![vec![0.0; n]; m];
    let mut basic = vec![vec![false; n]; m];
    let mut row_done = vec![false; m];
    let mut col_done = vec![false; n];
    fn assign(i: usize, j: usize, s: &mut [f64], d: &mut [f64], row_done: &mut [bool], col_done: &mut [bool], alloc: &mut [Vec<f64>], basic: &mut [Vec<bool>]) {
        let q = s[i].min(d[j]);
        alloc[i][j] = q;
        basic[i][j] = true;
        s[i] -= q;
        d[j] -= q;
        let open_rows = row_done.iter().filter(|x| !**x).count();
        if s[i] <= 1e-12 && open_rows > 1 {
            row_done[i] = true;
        } else {
            col_done[j] = true;
        }
    }
    match method {
        StartMethod::NorthWest => {
            let (mut i, mut j) = (0, 0);
            while i < m && j < n {
                assign(i, j, &mut s, &mut d, &mut row_done, &mut col_done, &mut alloc, &mut basic);
                if row_done[i] {
                    i += 1;
                } else {
                    j += 1;
                }
            }
        }
        StartMethod::LeastCost => loop {
            let mut best: Option<(usize, usize)> = None;
            for i in 0..m {
                if row_done[i] {
                    continue;
                }
                for j in 0..n {
                    if col_done[j] {
                        continue;
                    }
                    if best.map_or(true, |(bi, bj)| p.cost[i][j] < p.cost[bi][bj]) {
                        best = Some((i, j));
                    }
                }
            }
            match best {
                Some((i, j)) => assign(i, j, &mut s, &mut d, &mut row_done, &mut col_done, &mut alloc, &mut basic),
                None => break,
            }
        },
        StartMethod::Vogel => loop {
            let open_r: Vec<usize> = (0..m).filter(|&i| !row_done[i]).collect();
            let open_c: Vec<usize> = (0..n).filter(|&j| !col_done[j]).collect();
            if open_r.is_empty() || open_c.is_empty() {
                break;
            }
            if open_r.len() == 1 || open_c.len() == 1 {
                let mut cells: Vec<(usize, usize)> = Vec::new();
                for &i in &open_r {
                    for &j in &open_c {
                        cells.push((i, j));
                    }
                }
                cells.sort_by(|a, b| p.cost[a.0][a.1].partial_cmp(&p.cost[b.0][b.1]).unwrap());
                for (i, j) in cells {
                    if !row_done[i] && !col_done[j] {
                        assign(i, j, &mut s, &mut d, &mut row_done, &mut col_done, &mut alloc, &mut basic);
                    }
                }
                continue;
            }
            let penalty = |vals: &mut Vec<f64>| -> f64 {
                vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
                if vals.len() >= 2 {
                    vals[1] - vals[0]
                } else {
                    vals.first().copied().unwrap_or(0.0)
                }
            };
            let mut best_pen = -1.0;
            let mut pick: Option<(bool, usize)> = None;
            for &i in &open_r {
                let mut v: Vec<f64> = open_c.iter().map(|&j| p.cost[i][j]).collect();
                let pen = penalty(&mut v);
                if pen > best_pen + 1e-12 {
                    best_pen = pen;
                    pick = Some((true, i));
                }
            }
            for &j in &open_c {
                let mut v: Vec<f64> = open_r.iter().map(|&i| p.cost[i][j]).collect();
                let pen = penalty(&mut v);
                if pen > best_pen + 1e-12 {
                    best_pen = pen;
                    pick = Some((false, j));
                }
            }
            let (is_row, k) = pick.unwrap();
            let (i, j) = if is_row {
                let j = *open_c
                    .iter()
                    .min_by(|&&a, &&b| p.cost[k][a].partial_cmp(&p.cost[k][b]).unwrap())
                    .unwrap();
                (k, j)
            } else {
                let i = *open_r
                    .iter()
                    .min_by(|&&a, &&b| p.cost[a][k].partial_cmp(&p.cost[b][k]).unwrap())
                    .unwrap();
                (i, k)
            };
            assign(i, j, &mut s, &mut d, &mut row_done, &mut col_done, &mut alloc, &mut basic);
        },
    }
    complete_basis(p, &mut basic);
    (alloc, basic)
}

fn find(parent: &mut Vec<usize>, x: usize) -> usize {
    let mut r = x;
    while parent[r] != r {
        r = parent[r];
    }
    let mut y = x;
    while parent[y] != r {
        let nx = parent[y];
        parent[y] = r;
        y = nx;
    }
    r
}

fn complete_basis(p: &TransportProblem, basic: &mut Vec<Vec<bool>>) {
    let m = basic.len();
    let n = basic[0].len();
    let mut parent: Vec<usize> = (0..m + n).collect();
    let mut count = 0;
    for i in 0..m {
        for j in 0..n {
            if basic[i][j] {
                let a = find(&mut parent, i);
                let b = find(&mut parent, m + j);
                if a != b {
                    parent[a] = b;
                    count += 1;
                } else {
                    basic[i][j] = false;
                }
            }
        }
    }
    while count < m + n - 1 {
        let mut best: Option<(usize, usize)> = None;
        for i in 0..m {
            for j in 0..n {
                if basic[i][j] {
                    continue;
                }
                let a = find(&mut parent, i);
                let b = find(&mut parent, m + j);
                if a != b && best.map_or(true, |(bi, bj)| p.cost[i][j] < p.cost[bi][bj]) {
                    best = Some((i, j));
                }
            }
        }
        match best {
            Some((i, j)) => {
                basic[i][j] = true;
                let a = find(&mut parent, i);
                let b = find(&mut parent, m + j);
                parent[a] = b;
                count += 1;
            }
            None => break,
        }
    }
}

fn potentials(p: &TransportProblem, basic: &[Vec<bool>]) -> (Vec<f64>, Vec<f64>) {
    let m = basic.len();
    let n = basic[0].len();
    let mut u = vec![f64::NAN; m];
    let mut v = vec![f64::NAN; n];
    u[0] = 0.0;
    let mut changed = true;
    while changed {
        changed = false;
        for i in 0..m {
            for j in 0..n {
                if basic[i][j] {
                    if !u[i].is_nan() && v[j].is_nan() {
                        v[j] = p.cost[i][j] - u[i];
                        changed = true;
                    } else if u[i].is_nan() && !v[j].is_nan() {
                        u[i] = p.cost[i][j] - v[j];
                        changed = true;
                    }
                }
            }
        }
    }
    for x in u.iter_mut() {
        if x.is_nan() {
            *x = 0.0;
        }
    }
    for x in v.iter_mut() {
        if x.is_nan() {
            *x = 0.0;
        }
    }
    (u, v)
}

fn find_loop(basic: &[Vec<bool>], start: (usize, usize)) -> Vec<(usize, usize)> {
    let m = basic.len();
    let n = basic[0].len();
    let total = m + n;
    let mut prev: Vec<Option<(usize, (usize, usize))>> = vec![None; total];
    let mut seen = vec![false; total];
    let src = start.0;
    let dst = m + start.1;
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(src);
    seen[src] = true;
    while let Some(node) = queue.pop_front() {
        if node == dst {
            break;
        }
        if node < m {
            for j in 0..n {
                if basic[node][j] && !seen[m + j] {
                    seen[m + j] = true;
                    prev[m + j] = Some((node, (node, j)));
                    queue.push_back(m + j);
                }
            }
        } else {
            let j = node - m;
            for i in 0..m {
                if basic[i][j] && !seen[i] {
                    seen[i] = true;
                    prev[i] = Some((node, (i, j)));
                    queue.push_back(i);
                }
            }
        }
    }
    let mut path = Vec::new();
    let mut cur = dst;
    while cur != src {
        match prev[cur] {
            Some((p, cell)) => {
                path.push(cell);
                cur = p;
            }
            None => return vec![start],
        }
    }
    let mut cycle = vec![start];
    cycle.extend(path.into_iter());
    cycle
}

fn total_cost(p: &TransportProblem, alloc: &[Vec<f64>]) -> f64 {
    let mut t = 0.0;
    for i in 0..alloc.len() {
        for j in 0..alloc[0].len() {
            t += alloc[i][j] * p.cost[i][j];
        }
    }
    t
}

pub fn solve(original: &TransportProblem, method: StartMethod) -> Result<TransportSolution, String> {
    if original.supply.iter().any(|v| *v < 0.0) {
        return Err("Somente quantidades de oferta NÃO NEGATIVAS são permitidas.".into());
    }
    if original.demand.iter().any(|v| *v < 0.0) {
        return Err("Somente quantidades de demanda NÃO NEGATIVAS são permitidas.".into());
    }
    let p = balance(original);
    let (mut alloc, mut basic) = initial(&p, method);
    let m = p.supply.len();
    let n = p.demand.len();
    let mut tableaus = Vec::new();
    let mut optimal = false;
    for _ in 0..500 {
        let (u, v) = potentials(&p, &basic);
        let mut reduced = vec![vec![0.0; n]; m];
        let mut entering: Option<(usize, usize)> = None;
        let mut best = -1e-9;
        for i in 0..m {
            for j in 0..n {
                if !basic[i][j] {
                    let r = p.cost[i][j] - u[i] - v[j];
                    reduced[i][j] = r;
                    if r < best - 1e-12 {
                        best = r;
                        entering = Some((i, j));
                    }
                }
            }
        }
        let cost = total_cost(&p, &alloc);
        match entering {
            None => {
                tableaus.push(TransportTableau {
                    alloc: alloc.clone(),
                    basic: basic.clone(),
                    u,
                    v,
                    reduced,
                    entering: None,
                    loop_cells: vec![],
                    theta: 0.0,
                    leaving: None,
                    cost,
                });
                optimal = true;
                break;
            }
            Some(e) => {
                let cyc = find_loop(&basic, e);
                let mut theta = f64::INFINITY;
                let mut leaving = None;
                for (k, &(i, j)) in cyc.iter().enumerate() {
                    if k % 2 == 1 && alloc[i][j] < theta - 1e-12 {
                        theta = alloc[i][j];
                        leaving = Some((i, j));
                    }
                }
                let theta = if theta.is_finite() { theta } else { 0.0 };
                tableaus.push(TransportTableau {
                    alloc: alloc.clone(),
                    basic: basic.clone(),
                    u,
                    v,
                    reduced,
                    entering: Some(e),
                    loop_cells: cyc.clone(),
                    theta,
                    leaving,
                    cost,
                });
                for (k, &(i, j)) in cyc.iter().enumerate() {
                    if k % 2 == 0 {
                        alloc[i][j] += theta;
                    } else {
                        alloc[i][j] -= theta;
                        if alloc[i][j].abs() < 1e-12 {
                            alloc[i][j] = 0.0;
                        }
                    }
                }
                basic[e.0][e.1] = true;
                if let Some((li, lj)) = leaving {
                    basic[li][lj] = false;
                }
            }
        }
    }
    Ok(TransportSolution {
        problem: p,
        method,
        tableaus,
        optimal,
    })
}
