use crate::format::{num, parse_or_zero};
use crate::grid::{resize_grid, resize_vec};
use crate::linalg::gauss_jordan;
use crate::report::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct AlgebraInput {
    pub title: String,
    pub n: usize,
    pub a: Vec<Vec<String>>,
    pub b: Vec<String>,
}

impl AlgebraInput {
    pub fn blank(n: usize) -> Self {
        let mut s = Self {
            title: String::new(),
            n,
            a: vec![],
            b: vec![],
        };
        s.fit();
        s
    }

    pub fn fit(&mut self) {
        resize_grid(&mut self.a, self.n, self.n);
        resize_vec(&mut self.b, self.n, String::new());
    }

    pub fn example(with_rhs: bool) -> Self {
        let a = [["2", "1", "-1"], ["-3", "-1", "2"], ["-2", "1", "2"]];
        let b = ["8", "-11", "-3"];
        Self {
            title: if with_rhs { "Sistema 3x3 de exemplo".into() } else { "Matriz 3x3 de exemplo".into() },
            n: 3,
            a: a.iter().map(|r| r.iter().map(|s| s.to_string()).collect()).collect(),
            b: b.iter().map(|s| s.to_string()).collect(),
        }
    }
}

pub struct AlgebraOutcome {
    pub results: Vec<Section>,
    pub steps: Vec<Section>,
}

pub fn solve(input: &AlgebraInput, with_rhs: bool) -> AlgebraOutcome {
    let n = input.n;
    let a: Vec<Vec<f64>> = input.a.iter().map(|r| r.iter().map(|s| parse_or_zero(s)).collect()).collect();
    let b: Vec<f64> = input.b.iter().map(|s| parse_or_zero(s)).collect();
    let res = gauss_jordan(&a, if with_rhs { Some(&b) } else { None });
    let mut out = Vec::new();
    let mut orig = Section::new(if with_rhs { "Equações originais" } else { "Matriz original" });
    let mut h: Vec<String> = (1..=n).map(|j| format!("x{}", j)).collect();
    if with_rhs {
        h.push("LD".into());
    }
    orig = orig.headers_owned(std::iter::once(String::new()).chain(h).collect());
    for i in 0..n {
        let mut r = vec![cs(format!("linha {}", i + 1), CellStyle::Head)];
        r.extend(a[i].iter().map(|v| crate::report::n(*v)));
        if with_rhs {
            r.push(crate::report::n(b[i]));
        }
        orig.row(r);
    }
    out.push(orig);
    match &res.inverse {
        None => {
            let mut s = Section::new("Sem inversa");
            s.note("Matriz singular: a inversa não existe (determinante = 0).");
            out.push(s);
        }
        Some(inv) => {
            let mut det = Section::new("Determinante");
            det.line(format!("Valor do determinante = {}", num(res.determinant)));
            out.push(det);
            if let Some(x) = &res.solution {
                let mut s = Section::new("Solução").headers(&["Variável", "Valor"]);
                for (i, v) in x.iter().enumerate() {
                    s.row(vec![cs(format!("x{}", i + 1), CellStyle::Head), cs(num(*v), CellStyle::Bold)]);
                }
                out.push(s);
            }
            let mut s = Section::new("Matriz inversa")
                .headers_owned(std::iter::once(String::new()).chain((1..=n).map(|j| format!("col {}", j))).collect());
            for i in 0..n {
                let mut r = vec![cs(format!("linha {}", i + 1), CellStyle::Head)];
                r.extend(inv[i].iter().map(|v| crate::report::n(*v)));
                s.row(r);
            }
            out.push(s);
        }
    }
    let mut st = Section::new("Passos da eliminação (Gauss-Jordan com pivoteamento parcial)");
    for l in &res.steps {
        st.line(l.clone());
    }
    AlgebraOutcome { results: out, steps: vec![st] }
}

pub fn report(input: &AlgebraInput, with_rhs: bool, results: &[Section]) -> Option<String> {
    if results.is_empty() {
        return None;
    }
    let t = if with_rhs { "SOLUÇÃO DE EQUAÇÕES LINEARES SIMULTÂNEAS" } else { "INVERSÃO DE MATRIZ" };
    Some(sections_to_text(&format!("{}: {}", t, input.title), results))
}
