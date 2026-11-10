pub const EPSILON: f64 = 1e-8;
pub const BOUND_INFINITY: f64 = 999_999.0;
pub const BIG_M_DEFAULT: f64 = 100.0;
pub const OUTPUT_WIDTH: usize = 5;
pub const OUTPUT_DECIMALS: usize = 2;
pub const MAX_PROBLEM_SIZE: usize = 100;
pub const BB_MAX_SUBPROBLEMS: usize = 32_000;
pub const QUEUE_INFINITY: f64 = 9_999.0;
pub const QUEUE_PROB_CUTOFF: f64 = 1e-5;
pub const PERT_MODE_WEIGHT: f64 = 4.0;
pub const PERT_MEAN_DIVISOR: f64 = 6.0;
pub const PERT_VARIANCE_DIVISOR: f64 = 36.0;

pub fn is_integral(v: f64) -> bool {
    (v - (v + EPSILON).floor()).abs() < EPSILON
}

pub fn bound_value(v: f64) -> f64 {
    if v >= BOUND_INFINITY {
        f64::INFINITY
    } else if v <= -BOUND_INFINITY {
        f64::NEG_INFINITY
    } else {
        v
    }
}

pub fn queue_limit(v: f64) -> f64 {
    if v == QUEUE_INFINITY {
        f64::INFINITY
    } else {
        v
    }
}

pub fn pert_mean(a: f64, m: f64, b: f64) -> f64 {
    (m * PERT_MODE_WEIGHT + a + b) / PERT_MEAN_DIVISOR
}

pub fn pert_variance(a: f64, b: f64) -> f64 {
    let d = b - a;
    d * d / PERT_VARIANCE_DIVISOR
}
