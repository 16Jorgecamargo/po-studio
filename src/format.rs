use std::cell::Cell;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct NumFormat {
    pub scientific: bool,
    pub width: usize,
    pub decimals: usize,
}

impl Default for NumFormat {
    fn default() -> Self {
        Self {
            scientific: false,
            width: crate::mechanics::OUTPUT_WIDTH,
            decimals: crate::mechanics::OUTPUT_DECIMALS,
        }
    }
}

thread_local! {
    static CURRENT: Cell<NumFormat> = Cell::new(NumFormat::default());
}

pub fn set_format(f: NumFormat) {
    CURRENT.with(|c| c.set(f));
}

pub fn get_format() -> NumFormat {
    CURRENT.with(|c| c.get())
}

pub fn num(v: f64) -> String {
    let f = get_format();
    if v.is_infinite() {
        return if v > 0.0 { "∞".into() } else { "-∞".into() };
    }
    if v.is_nan() {
        return "–".into();
    }
    let v = if v.abs() < 0.5 * 10f64.powi(-(f.decimals as i32)) { 0.0 } else { v };
    if f.scientific {
        format!("{:.*e}", f.decimals, v)
    } else {
        format!("{:.*}", f.decimals, v)
    }
}

pub fn parse_num(s: &str) -> Option<f64> {
    let t = s.trim().replace(',', ".");
    if t.is_empty() {
        return None;
    }
    let lower = t.to_lowercase();
    match lower.as_str() {
        "inf" | "infinity" | "infinito" | "∞" | "+inf" => return Some(f64::INFINITY),
        "-inf" | "-infinity" | "-infinito" | "-∞" => return Some(f64::NEG_INFINITY),
        _ => {}
    }
    t.parse::<f64>().ok()
}

pub fn parse_or_zero(s: &str) -> f64 {
    parse_num(s).unwrap_or(0.0)
}

pub fn yes(s: &str) -> bool {
    matches!(s.trim().to_lowercase().as_str(), "y" | "s" | "sim" | "yes" | "1" | "x")
}
