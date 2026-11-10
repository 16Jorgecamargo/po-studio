use crate::lp::{lp_status_only, LpModel};
use crate::simplex::{Rel, Sense, Status};

#[derive(Clone, Debug)]
pub struct GameResult {
    pub value: f64,
    pub a_mix: Vec<f64>,
    pub b_mix: Vec<f64>,
    pub saddle: Option<(usize, usize)>,
    pub maximin: f64,
    pub minimax: f64,
    pub shift: f64,
    pub a_lp: LpModel,
}

pub fn solve(payoff: &[Vec<f64>]) -> Result<GameResult, String> {
    let m = payoff.len();
    let n = payoff.first().map(|r| r.len()).unwrap_or(0);
    if m == 0 || n == 0 {
        return Err("Informe a matriz de pagamentos.".into());
    }
    let row_min: Vec<f64> = payoff.iter().map(|r| r.iter().cloned().fold(f64::INFINITY, f64::min)).collect();
    let col_max: Vec<f64> = (0..n)
        .map(|j| (0..m).map(|i| payoff[i][j]).fold(f64::NEG_INFINITY, f64::max))
        .collect();
    let maximin = row_min.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let minimax = col_max.iter().cloned().fold(f64::INFINITY, f64::min);
    let mut saddle = None;
    if (maximin - minimax).abs() < 1e-12 {
        let i = row_min.iter().position(|&v| v == maximin).unwrap();
        let j = col_max.iter().position(|&v| v == minimax).unwrap();
        saddle = Some((i, j));
    }
    let min_all = payoff.iter().flat_map(|r| r.iter()).cloned().fold(f64::INFINITY, f64::min);
    let shift = if min_all <= 0.0 { 1.0 - min_all } else { 0.0 };
    let a_lp = LpModel {
        sense: Sense::Min,
        var_names: (1..=m).map(|i| format!("X{}", i)).collect(),
        c: vec![1.0; m],
        a: (0..n).map(|j| (0..m).map(|i| payoff[i][j] + shift).collect()).collect(),
        rel: vec![Rel::Ge; n],
        b: vec![1.0; n],
        con_names: (1..=n).map(|j| format!("B{}", j)).collect(),
        lower: vec![0.0; m],
        upper: vec![f64::INFINITY; m],
        integer: vec![false; m],
    };
    let b_lp = LpModel {
        sense: Sense::Max,
        var_names: (1..=n).map(|j| format!("Y{}", j)).collect(),
        c: vec![1.0; n],
        a: (0..m).map(|i| (0..n).map(|j| payoff[i][j] + shift).collect()).collect(),
        rel: vec![Rel::Le; m],
        b: vec![1.0; m],
        con_names: (1..=m).map(|i| format!("A{}", i)).collect(),
        lower: vec![0.0; n],
        upper: vec![f64::INFINITY; n],
        integer: vec![false; n],
    };
    let (sa, za, xa) = lp_status_only(&a_lp).ok_or("Falha ao resolver o PL do jogador A.")?;
    let (sb, _zb, yb) = lp_status_only(&b_lp).ok_or("Falha ao resolver o PL do jogador B.")?;
    if sa != Status::Optimal || sb != Status::Optimal || za <= 0.0 {
        return Err("Não foi possível resolver o jogo.".into());
    }
    let v = 1.0 / za;
    let a_mix: Vec<f64> = xa.iter().map(|x| x * v).collect();
    let b_mix: Vec<f64> = yb.iter().map(|y| y * v).collect();
    Ok(GameResult {
        value: v - shift,
        a_mix,
        b_mix,
        saddle,
        maximin,
        minimax,
        shift,
        a_lp,
    })
}
