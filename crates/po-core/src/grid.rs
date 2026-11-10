pub fn resize_vec<T: Clone>(v: &mut Vec<T>, n: usize, fill: T) {
    v.resize(n, fill);
}

pub fn resize_grid(g: &mut Vec<Vec<String>>, rows: usize, cols: usize) {
    g.resize(rows, vec![String::new(); cols]);
    for r in g.iter_mut() {
        r.resize(cols, String::new());
    }
}

pub fn resize_names(v: &mut Vec<String>, n: usize, prefix: &str) {
    let start = v.len();
    v.truncate(n);
    for i in start..n {
        v.push(format!("{}{}", prefix, i + 1));
    }
}
