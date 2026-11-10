pub const INF: f64 = f64::INFINITY;

#[derive(Clone, Debug)]
pub struct MstStep {
    pub from: usize,
    pub to: usize,
    pub length: f64,
    pub total: f64,
}

pub fn minimal_spanning_tree(dist: &[Vec<f64>], start: usize) -> Result<Vec<MstStep>, String> {
    let n = dist.len();
    let w = |i: usize, j: usize| -> f64 {
        let a = dist[i][j];
        let b = dist[j][i];
        a.min(b)
    };
    let mut in_tree = vec![false; n];
    in_tree[start] = true;
    let mut steps = Vec::new();
    let mut total = 0.0;
    for _ in 1..n {
        let mut best: Option<(usize, usize, f64)> = None;
        for i in 0..n {
            if !in_tree[i] {
                continue;
            }
            for j in 0..n {
                if in_tree[j] || i == j {
                    continue;
                }
                let d = w(i, j);
                if d.is_finite() && best.map_or(true, |(_, _, bd)| d < bd) {
                    best = Some((i, j, d));
                }
            }
        }
        match best {
            Some((i, j, d)) => {
                in_tree[j] = true;
                total += d;
                steps.push(MstStep {
                    from: i,
                    to: j,
                    length: d,
                    total,
                });
            }
            None => {
                return Err("Todos os arcos a partir dos nós atuais têm comprimento infinito. Não existe árvore geradora.".into());
            }
        }
    }
    Ok(steps)
}

#[derive(Clone, Debug)]
pub struct DijkstraIter {
    pub permanent: usize,
    pub labels: Vec<Option<(f64, usize)>>,
    pub is_permanent: Vec<bool>,
}

pub fn dijkstra(dist: &[Vec<f64>], source: usize) -> (Vec<f64>, Vec<Option<usize>>, Vec<DijkstraIter>) {
    let n = dist.len();
    let mut d = vec![INF; n];
    let mut pred: Vec<Option<usize>> = vec![None; n];
    let mut perm = vec![false; n];
    let mut labels: Vec<Option<(f64, usize)>> = vec![None; n];
    d[source] = 0.0;
    labels[source] = Some((0.0, source));
    let mut iters = Vec::new();
    loop {
        let mut u: Option<usize> = None;
        for i in 0..n {
            if !perm[i] && d[i].is_finite() && u.map_or(true, |k| d[i] < d[k]) {
                u = Some(i);
            }
        }
        let Some(u) = u else { break };
        perm[u] = true;
        for v in 0..n {
            if v != u && !perm[v] && dist[u][v].is_finite() {
                let nd = d[u] + dist[u][v];
                if nd < d[v] - 1e-12 {
                    d[v] = nd;
                    pred[v] = Some(u);
                    labels[v] = Some((nd, u));
                }
            }
        }
        iters.push(DijkstraIter {
            permanent: u,
            labels: labels.clone(),
            is_permanent: perm.clone(),
        });
    }
    (d, pred, iters)
}

pub fn route(pred: &[Option<usize>], source: usize, target: usize) -> Option<Vec<usize>> {
    let mut r = vec![target];
    let mut cur = target;
    let mut guard = 0;
    while cur != source {
        cur = pred[cur]?;
        r.push(cur);
        guard += 1;
        if guard > pred.len() + 1 {
            return None;
        }
    }
    r.reverse();
    Some(r)
}

#[derive(Clone, Debug)]
pub struct FloydIter {
    pub k: usize,
    pub d: Vec<Vec<f64>>,
    pub s: Vec<Vec<usize>>,
}

pub fn floyd(dist: &[Vec<f64>]) -> Vec<FloydIter> {
    let n = dist.len();
    let mut d: Vec<Vec<f64>> = (0..n)
        .map(|i| (0..n).map(|j| if i == j { 0.0 } else { dist[i][j] }).collect())
        .collect();
    let mut s: Vec<Vec<usize>> = (0..n).map(|_| (0..n).collect()).collect();
    let mut out = vec![FloydIter {
        k: 0,
        d: d.clone(),
        s: s.clone(),
    }];
    for k in 0..n {
        for i in 0..n {
            if i == k || !d[i][k].is_finite() {
                continue;
            }
            for j in 0..n {
                if j == k || i == j {
                    continue;
                }
                let nd = d[i][k] + d[k][j];
                if nd < d[i][j] - 1e-12 {
                    d[i][j] = nd;
                    s[i][j] = k;
                }
            }
        }
        out.push(FloydIter {
            k: k + 1,
            d: d.clone(),
            s: s.clone(),
        });
    }
    out
}

pub fn floyd_route(s: &[Vec<usize>], i: usize, j: usize) -> Vec<usize> {
    fn walk(s: &[Vec<usize>], i: usize, j: usize, depth: usize, out: &mut Vec<usize>) {
        let k = s[i][j];
        if k == j || depth > s.len() {
            out.push(j);
        } else {
            walk(s, i, k, depth + 1, out);
            walk(s, k, j, depth + 1, out);
        }
    }
    let mut out = vec![i];
    walk(s, i, j, 0, &mut out);
    out
}

#[derive(Clone, Debug)]
pub struct FlowIter {
    pub path: Vec<usize>,
    pub flow: f64,
    pub total: f64,
}

pub struct MaxFlowResult {
    pub total: f64,
    pub flows: Vec<Vec<f64>>,
    pub iters: Vec<FlowIter>,
}

pub fn max_flow(cap: &[Vec<f64>], source: usize, sink: usize) -> MaxFlowResult {
    let n = cap.len();
    let mut residual: Vec<Vec<f64>> = cap
        .iter()
        .map(|r| r.iter().map(|v| if v.is_finite() { *v } else { 1e15 }).collect())
        .collect();
    let mut iters = Vec::new();
    let mut total = 0.0;
    loop {
        let mut prev = vec![usize::MAX; n];
        prev[source] = source;
        let mut q = std::collections::VecDeque::new();
        q.push_back(source);
        while let Some(u) = q.pop_front() {
            if u == sink {
                break;
            }
            for v in 0..n {
                if prev[v] == usize::MAX && residual[u][v] > 1e-12 {
                    prev[v] = u;
                    q.push_back(v);
                }
            }
        }
        if prev[sink] == usize::MAX {
            break;
        }
        let mut path = vec![sink];
        let mut f = f64::INFINITY;
        let mut v = sink;
        while v != source {
            let u = prev[v];
            f = f.min(residual[u][v]);
            path.push(u);
            v = u;
        }
        path.reverse();
        let mut v = sink;
        while v != source {
            let u = prev[v];
            residual[u][v] -= f;
            residual[v][u] += f;
            v = u;
        }
        total += f;
        iters.push(FlowIter { path, flow: f, total });
        if iters.len() > 10000 {
            break;
        }
    }
    let mut flows = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..n {
            let c = if cap[i][j].is_finite() { cap[i][j] } else { 1e15 };
            let f = c - residual[i][j];
            if f > 1e-12 && cap[i][j] > 0.0 {
                flows[i][j] = f.min(c);
            }
        }
    }
    for i in 0..n {
        for j in i + 1..n {
            let net = flows[i][j] - flows[j][i];
            if net >= 0.0 {
                flows[i][j] = net;
                flows[j][i] = 0.0;
            } else {
                flows[j][i] = -net;
                flows[i][j] = 0.0;
            }
        }
    }
    MaxFlowResult { total, flows, iters }
}
