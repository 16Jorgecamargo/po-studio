#[derive(Clone, Debug)]
pub struct QueueInput {
    pub name: String,
    pub lambda: f64,
    pub mu: f64,
    pub servers: usize,
    pub system_limit: f64,
    pub source_limit: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum QueueKind {
    General,
    PureBirth,
    PureDeath,
}

#[derive(Clone, Debug)]
pub struct QueueResult {
    pub input: QueueInput,
    pub kind: QueueKind,
    pub label: String,
    pub lambda_eff: f64,
    pub rho_c: f64,
    pub ls: f64,
    pub lq: f64,
    pub ws: f64,
    pub wq: f64,
    pub probs: Vec<f64>,
}

const TAIL: f64 = crate::mechanics::QUEUE_PROB_CUTOFF;
const MAX_STATES: usize = 2000;

fn fmt_limit(v: f64) -> String {
    if v.is_infinite() {
        "∞".into()
    } else {
        format!("{}", v as i64)
    }
}

pub fn analyse(q: &QueueInput) -> Result<QueueResult, String> {
    let err = || {
        format!(
            "Dados inconsistentes no cenário {}. Verifique: (1) valores negativos, (2) Lambda e Mu ambos = 0, (3) Lambda >= Mu·c em fila infinita, (4) limite do sistema maior que o limite da fonte, (5) N ausente ou zero no modelo de morte pura.",
            q.name
        )
    };
    if q.lambda < 0.0 || q.mu < 0.0 || (q.lambda == 0.0 && q.mu == 0.0) {
        return Err(err());
    }
    if q.mu == 0.0 {
        let lt = q.lambda;
        let mut probs = Vec::new();
        let mut p = (-lt).exp();
        let mut cum = 0.0;
        let mut n = 0usize;
        while n < MAX_STATES {
            probs.push(p);
            cum += p;
            if n as f64 > lt && (p < TAIL || cum > 1.0 - 1e-12) {
                break;
            }
            n += 1;
            p *= lt / n as f64;
        }
        let mean = lt;
        return Ok(QueueResult {
            input: q.clone(),
            kind: QueueKind::PureBirth,
            label: format!("{}: Modelo de nascimento puro (λt = {})", q.name, crate::format::num(lt)),
            lambda_eff: q.lambda,
            rho_c: f64::NAN,
            ls: mean,
            lq: f64::NAN,
            ws: f64::NAN,
            wq: f64::NAN,
            probs,
        });
    }
    if q.lambda == 0.0 {
        if !q.system_limit.is_finite() || q.system_limit <= 0.0 {
            return Err(err());
        }
        let nmax = q.system_limit as usize;
        let mt = q.mu;
        let mut probs = vec![0.0; nmax + 1];
        let mut tail_sum = 0.0;
        for n in (1..=nmax).rev() {
            let k = (nmax - n) as i32;
            let mut v = (-mt).exp();
            for i in 1..=k {
                v *= mt / i as f64;
            }
            probs[n] = v;
            tail_sum += v;
        }
        probs[0] = (1.0 - tail_sum).max(0.0);
        let ls: f64 = probs.iter().enumerate().map(|(n, p)| n as f64 * p).sum();
        return Ok(QueueResult {
            input: q.clone(),
            kind: QueueKind::PureDeath,
            label: format!("{}: Modelo de morte pura (N = {}, μt = {})", q.name, nmax, crate::format::num(mt)),
            lambda_eff: 0.0,
            rho_c: f64::NAN,
            ls,
            lq: f64::NAN,
            ws: f64::NAN,
            wq: f64::NAN,
            probs,
        });
    }
    let c = q.servers.max(1);
    let n_lim = q.system_limit;
    let k_lim = q.source_limit;
    if n_lim.is_finite() && k_lim.is_finite() && n_lim > k_lim {
        return Err(err());
    }
    let cap = if n_lim.is_finite() {
        n_lim as usize
    } else if k_lim.is_finite() {
        k_lim as usize
    } else {
        usize::MAX
    };
    if cap == usize::MAX && q.lambda >= q.mu * c as f64 {
        return Err(err());
    }
    if cap < c && cap != usize::MAX && n_lim.is_finite() && (n_lim as usize) < c {
        return Err(err());
    }
    let lam_n = |n: usize| -> f64 {
        if n >= cap {
            0.0
        } else if k_lim.is_finite() {
            (k_lim - n as f64).max(0.0) * q.lambda
        } else {
            q.lambda
        }
    };
    let mu_n = |n: usize| -> f64 { (n.min(c)) as f64 * q.mu };
    let mut rel = vec![1.0f64];
    let limit = if cap == usize::MAX { MAX_STATES * 50 } else { cap.min(MAX_STATES * 50) };
    let mut n = 0usize;
    while n < limit {
        let next = rel[n] * lam_n(n) / mu_n(n + 1);
        rel.push(next);
        n += 1;
        if cap == usize::MAX && n > c && next < 1e-14 * rel.iter().sum::<f64>() {
            break;
        }
    }
    let total: f64 = rel.iter().sum();
    let probs_all: Vec<f64> = rel.iter().map(|r| r / total).collect();
    let ls: f64 = probs_all.iter().enumerate().map(|(n, p)| n as f64 * p).sum();
    let lq: f64 = probs_all
        .iter()
        .enumerate()
        .map(|(n, p)| if n > c { (n - c) as f64 * p } else { 0.0 })
        .sum();
    let lambda_eff: f64 = probs_all.iter().enumerate().map(|(n, p)| lam_n(n) * p).sum();
    let ws = if lambda_eff > 0.0 { ls / lambda_eff } else { f64::NAN };
    let wq = if lambda_eff > 0.0 { lq / lambda_eff } else { f64::NAN };
    let mut probs = Vec::new();
    let mut cum = 0.0;
    for p in &probs_all {
        probs.push(*p);
        cum += p;
        if cap == usize::MAX && probs.len() > c + 1 && (*p < TAIL || cum > 1.0 - 1e-12) {
            break;
        }
        if probs.len() >= MAX_STATES {
            break;
        }
    }
    let label = format!(
        "{}: (M/M/{}):(GD/{}/{})",
        q.name,
        c,
        fmt_limit(n_lim),
        fmt_limit(k_lim)
    );
    Ok(QueueResult {
        input: q.clone(),
        kind: QueueKind::General,
        label,
        lambda_eff,
        rho_c: q.lambda / (q.mu * c as f64),
        ls,
        lq,
        ws,
        wq,
        probs,
    })
}
