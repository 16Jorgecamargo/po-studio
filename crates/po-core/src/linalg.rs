pub const SINGULAR_EPS: f64 = 1e-10;

pub struct GaussResult {
    pub determinant: f64,
    pub inverse: Option<Vec<Vec<f64>>>,
    pub solution: Option<Vec<f64>>,
    pub steps: Vec<String>,
}

pub fn gauss_jordan(a: &[Vec<f64>], b: Option<&[f64]>) -> GaussResult {
    let n = a.len();
    let mut m: Vec<Vec<f64>> = a.iter().map(|r| r.clone()).collect();
    let mut inv: Vec<Vec<f64>> = (0..n)
        .map(|i| (0..n).map(|j| if i == j { 1.0 } else { 0.0 }).collect())
        .collect();
    let mut rhs: Vec<f64> = b.map(|v| v.to_vec()).unwrap_or_else(|| vec![0.0; n]);
    let scale = a
        .iter()
        .flat_map(|r| r.iter())
        .fold(0.0f64, |acc, v| acc.max(v.abs()))
        .max(1.0);
    let mut det = 1.0;
    let mut steps = Vec::new();
    for col in 0..n {
        let mut piv = col;
        for r in col + 1..n {
            if m[r][col].abs() > m[piv][col].abs() {
                piv = r;
            }
        }
        if m[piv][col].abs() < SINGULAR_EPS * scale {
            return GaussResult {
                determinant: 0.0,
                inverse: None,
                solution: None,
                steps,
            };
        }
        if piv != col {
            m.swap(piv, col);
            inv.swap(piv, col);
            rhs.swap(piv, col);
            det = -det;
            steps.push(format!("Troca linha {} ↔ linha {}", col + 1, piv + 1));
        }
        let p = m[col][col];
        det *= p;
        for j in 0..n {
            m[col][j] /= p;
            inv[col][j] /= p;
        }
        rhs[col] /= p;
        for r in 0..n {
            if r != col {
                let f = m[r][col];
                if f != 0.0 {
                    for j in 0..n {
                        m[r][j] -= f * m[col][j];
                        inv[r][j] -= f * inv[col][j];
                    }
                    rhs[r] -= f * rhs[col];
                }
            }
        }
        steps.push(format!("Pivô na coluna {} = {}", col + 1, crate::format::num(p)));
    }
    GaussResult {
        determinant: det,
        inverse: Some(inv),
        solution: b.map(|_| rhs),
        steps,
    }
}
