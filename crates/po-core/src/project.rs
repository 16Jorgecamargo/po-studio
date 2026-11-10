use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct Activity {
    pub name: String,
    pub from: usize,
    pub to: usize,
    pub duration: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActivitySchedule {
    pub name: String,
    pub from: usize,
    pub to: usize,
    pub duration: f64,
    pub es: f64,
    pub ef: f64,
    pub ls: f64,
    pub lf: f64,
    pub total_float: f64,
    pub free_float: f64,
    pub critical: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct CpmResult {
    pub nodes: Vec<usize>,
    pub early: Vec<f64>,
    pub late: Vec<f64>,
    pub activities: Vec<ActivitySchedule>,
    pub duration: f64,
    pub forward_steps: Vec<String>,
    pub backward_steps: Vec<String>,
}

pub fn cpm(acts: &[Activity]) -> Result<CpmResult, String> {
    if acts.is_empty() {
        return Err("Informe ao menos uma atividade.".into());
    }
    for a in acts {
        if a.from < 1 || a.to < 1 {
            return Err("O número do nó deve ser >= 1.".into());
        }
        if a.from == a.to {
            return Err(format!("Os nós DE e PARA da atividade {} devem ter números diferentes.", a.name));
        }
        if a.duration < 0.0 {
            return Err(format!("Duração negativa na atividade {}.", a.name));
        }
    }
    for (i, a) in acts.iter().enumerate() {
        for b in acts.iter().skip(i + 1) {
            if a.from == b.from && a.to == b.to {
                return Err(format!(
                    "Os nós que definem a atividade {} já foram usados pela atividade {}.",
                    b.name, a.name
                ));
            }
        }
    }
    let mut nodes: Vec<usize> = acts.iter().flat_map(|a| [a.from, a.to]).collect();
    nodes.sort();
    nodes.dedup();
    let idx = |n: usize| nodes.iter().position(|&x| x == n).unwrap();
    let k = nodes.len();
    let starts: Vec<usize> = nodes.iter().copied().filter(|&n| !acts.iter().any(|a| a.to == n)).collect();
    let finishes: Vec<usize> = nodes.iter().copied().filter(|&n| !acts.iter().any(|a| a.from == n)).collect();
    if starts.len() > 1 {
        return Err(format!("INÍCIO MÚLTIPLO detectado nos nós ({}).", join(&starts)));
    }
    if finishes.len() > 1 {
        return Err(format!("TÉRMINO MÚLTIPLO detectado nos nós ({}).", join(&finishes)));
    }
    if starts.is_empty() || finishes.is_empty() {
        return Err("Laço detectado na rede.".into());
    }
    let mut indeg = vec![0usize; k];
    for a in acts {
        indeg[idx(a.to)] += 1;
    }
    let mut order = Vec::new();
    let mut stack: Vec<usize> = (0..k).filter(|&i| indeg[i] == 0).collect();
    while let Some(u) = stack.pop() {
        order.push(u);
        for a in acts.iter().filter(|a| idx(a.from) == u) {
            let v = idx(a.to);
            indeg[v] -= 1;
            if indeg[v] == 0 {
                stack.push(v);
            }
        }
    }
    if order.len() < k {
        let stuck = (0..k).find(|&i| indeg[i] > 0).map(|i| nodes[i]).unwrap_or(0);
        return Err(format!("Laço detectado. A passagem para frente não pode passar do nó {}.", stuck));
    }
    let mut early = vec![0.0; k];
    let mut forward_steps = Vec::new();
    for &u in &order {
        let incoming: Vec<&Activity> = acts.iter().filter(|a| idx(a.to) == u).collect();
        if incoming.is_empty() {
            forward_steps.push(format!("Nó {}: tempo mais cedo = 0", nodes[u]));
            continue;
        }
        let mut best = f64::NEG_INFINITY;
        let mut parts = Vec::new();
        for a in incoming {
            let v = early[idx(a.from)] + a.duration;
            parts.push(format!("{}+{}", crate::format::num(early[idx(a.from)]), crate::format::num(a.duration)));
            best = best.max(v);
        }
        early[u] = best;
        forward_steps.push(format!(
            "Nó {}: max{{{}}} = {}",
            nodes[u],
            parts.join(", "),
            crate::format::num(best)
        ));
    }
    let project = early.iter().cloned().fold(0.0, f64::max);
    let mut late = vec![project; k];
    let mut backward_steps = Vec::new();
    for &u in order.iter().rev() {
        let outgoing: Vec<&Activity> = acts.iter().filter(|a| idx(a.from) == u).collect();
        if outgoing.is_empty() {
            backward_steps.push(format!("Nó {}: tempo mais tarde = {}", nodes[u], crate::format::num(project)));
            continue;
        }
        let mut best = f64::INFINITY;
        let mut parts = Vec::new();
        for a in outgoing {
            let v = late[idx(a.to)] - a.duration;
            parts.push(format!("{}-{}", crate::format::num(late[idx(a.to)]), crate::format::num(a.duration)));
            best = best.min(v);
        }
        late[u] = best;
        backward_steps.push(format!(
            "Nó {}: min{{{}}} = {}",
            nodes[u],
            parts.join(", "),
            crate::format::num(best)
        ));
    }
    let activities = acts
        .iter()
        .map(|a| {
            let i = idx(a.from);
            let j = idx(a.to);
            let es = early[i];
            let lf = late[j];
            let tf = lf - es - a.duration;
            let ff = early[j] - es - a.duration;
            ActivitySchedule {
                name: a.name.clone(),
                from: a.from,
                to: a.to,
                duration: a.duration,
                es,
                ef: es + a.duration,
                ls: lf - a.duration,
                lf,
                total_float: clean(tf),
                free_float: clean(ff),
                critical: tf.abs() < 1e-9,
            }
        })
        .collect();
    Ok(CpmResult {
        nodes,
        early,
        late,
        activities,
        duration: project,
        forward_steps,
        backward_steps,
    })
}

fn clean(v: f64) -> f64 {
    if v.abs() < 1e-9 {
        0.0
    } else {
        v
    }
}

fn join(v: &[usize]) -> String {
    v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ")
}

#[derive(Clone, Debug)]
pub struct PertActivity {
    pub name: String,
    pub from: usize,
    pub to: usize,
    pub a: f64,
    pub m: f64,
    pub b: f64,
}

#[derive(Clone, Debug)]
pub struct PertResult {
    pub means: Vec<f64>,
    pub variances: Vec<f64>,
    pub cpm: CpmResult,
    pub path: Vec<usize>,
    pub path_mean: f64,
    pub path_std: f64,
}

pub fn pert(acts: &[PertActivity]) -> Result<PertResult, String> {
    for p in acts {
        if !(p.a <= p.m && p.m <= p.b) {
            return Err(format!("As durações da atividade {} devem satisfazer a <= m <= b.", p.name));
        }
    }
    let means: Vec<f64> = acts.iter().map(|p| crate::mechanics::pert_mean(p.a, p.m, p.b)).collect();
    let variances: Vec<f64> = acts.iter().map(|p| crate::mechanics::pert_variance(p.a, p.b)).collect();
    let base: Vec<Activity> = acts
        .iter()
        .zip(means.iter())
        .map(|(p, &d)| Activity {
            name: p.name.clone(),
            from: p.from,
            to: p.to,
            duration: d,
        })
        .collect();
    let cpm = cpm(&base)?;
    let start = *cpm.nodes.iter().find(|&&n| !acts.iter().any(|a| a.to == n)).unwrap();
    let finish = *cpm.nodes.iter().find(|&&n| !acts.iter().any(|a| a.from == n)).unwrap();
    let mut path = Vec::new();
    let mut cur = start;
    let mut guard = 0;
    while cur != finish && guard < acts.len() + 1 {
        let next = cpm
            .activities
            .iter()
            .enumerate()
            .filter(|(_, s)| s.from == cur && s.critical)
            .max_by(|a, b| variances[a.0].partial_cmp(&variances[b.0]).unwrap());
        match next {
            Some((i, s)) => {
                path.push(i);
                cur = s.to;
            }
            None => break,
        }
        guard += 1;
    }
    let path_mean: f64 = path.iter().map(|&i| means[i]).sum();
    let path_std: f64 = path.iter().map(|&i| variances[i]).sum::<f64>().sqrt();
    Ok(PertResult {
        means,
        variances,
        cpm,
        path,
        path_mean,
        path_std,
    })
}

pub fn normal_cdf(z: f64) -> f64 {
    0.5 * (1.0 + erf(z / std::f64::consts::SQRT_2))
}

fn erf(x: f64) -> f64 {
    let t = 1.0 / (1.0 + 0.3275911 * x.abs());
    let y = 1.0
        - (((((1.061405429 * t - 1.453152027) * t) + 1.421413741) * t - 0.284496736) * t + 0.254829592)
            * t
            * (-x * x).exp();
    if x >= 0.0 {
        y
    } else {
        -y
    }
}
