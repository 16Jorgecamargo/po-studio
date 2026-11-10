use crate::lp::{lp_status_only, LpModel};
use crate::simplex::{Sense, Status};
use serde::Serialize;

pub const MAX_SUBPROBLEMS: usize = crate::mechanics::BB_MAX_SUBPROBLEMS;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeStatus {
    Infeasible,
    Unbounded,
    Integer,
    Fathomed,
    Branched,
}

#[derive(Clone, Debug, Serialize)]
pub struct BbNode {
    pub id: usize,
    pub parent: Option<usize>,
    pub depth: usize,
    pub branch: String,
    pub z: Option<f64>,
    pub x: Vec<f64>,
    pub status: NodeStatus,
    pub branch_var: Option<usize>,
    pub improved: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct BbResult {
    pub nodes: Vec<BbNode>,
    pub best: Option<usize>,
    pub aborted: bool,
    pub improvements: Vec<usize>,
}

fn is_better(sense: Sense, a: f64, b: f64) -> bool {
    match sense {
        Sense::Max => a > b + 1e-9,
        Sense::Min => a < b - 1e-9,
    }
}

pub fn branch_and_bound(model: &LpModel) -> BbResult {
    let mut nodes: Vec<BbNode> = Vec::new();
    let mut best: Option<usize> = None;
    let mut best_z: Option<f64> = None;
    let mut improvements = Vec::new();
    let mut stack: Vec<(Option<usize>, usize, String, Vec<f64>, Vec<f64>)> =
        vec![(None, 0, "LP contínuo".into(), model.lower.clone(), model.upper.clone())];
    let mut aborted = false;
    while let Some((parent, depth, branch, lo, hi)) = stack.pop() {
        if nodes.len() >= MAX_SUBPROBLEMS {
            aborted = true;
            break;
        }
        let id = nodes.len() + 1;
        let mut sub = model.clone();
        sub.lower = lo.clone();
        sub.upper = hi.clone();
        let infeasible_bounds = (0..lo.len()).any(|j| lo[j] > hi[j] + 1e-9);
        let res = if infeasible_bounds { None } else { lp_status_only(&sub) };
        let mut node = BbNode {
            id,
            parent,
            depth,
            branch,
            z: None,
            x: vec![],
            status: NodeStatus::Infeasible,
            branch_var: None,
            improved: false,
        };
        match res {
            None | Some((Status::Infeasible, _, _)) => {
                node.status = NodeStatus::Infeasible;
            }
            Some((Status::Unbounded, _, x)) => {
                node.x = x;
                node.status = NodeStatus::Unbounded;
            }
            Some((Status::Optimal, z, x)) => {
                node.z = Some(z);
                node.x = x.clone();
                if let Some(bz) = best_z {
                    if !is_better(model.sense, z, bz) {
                        node.status = NodeStatus::Fathomed;
                        nodes.push(node);
                        continue;
                    }
                }
                let frac = (0..x.len()).find(|&j| model.integer[j] && !crate::mechanics::is_integral(x[j]));
                match frac {
                    None => {
                        node.status = NodeStatus::Integer;
                        node.improved = true;
                        best = Some(id);
                        best_z = Some(z);
                        improvements.push(id);
                    }
                    Some(j) => {
                        node.status = NodeStatus::Branched;
                        node.branch_var = Some(j);
                        let fl = x[j].floor();
                        let ce = x[j].ceil();
                        let mut hi2 = hi.clone();
                        hi2[j] = hi2[j].min(fl);
                        let mut lo2 = lo.clone();
                        lo2[j] = lo2[j].max(ce);
                        let name = &model.var_names[j];
                        stack.push((Some(id), depth + 1, format!("{} >= {}", name, ce), lo2, hi.clone()));
                        stack.push((Some(id), depth + 1, format!("{} <= {}", name, fl), lo.clone(), hi2));
                    }
                }
            }
        }
        nodes.push(node);
    }
    BbResult {
        nodes,
        best,
        aborted,
        improvements,
    }
}
