use serde_json::{json, Value};

pub struct Imported {
    pub module: &'static str,
    pub data: Value,
}

pub fn looks_legacy(text: &str) -> bool {
    let first = text.lines().map(str::trim).find(|l| !l.is_empty()).unwrap_or("");
    let token = first.split(',').next().unwrap_or("").trim();
    token.parse::<i64>().map(|c| (1000..10000).contains(&c)).unwrap_or(false)
}

fn tokenize(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim_end_matches(['\r', '\n']);
        if line.trim().is_empty() {
            continue;
        }
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            while i < chars.len() && chars[i] == ' ' {
                i += 1;
            }
            if i >= chars.len() {
                break;
            }
            if chars[i] == '"' {
                i += 1;
                let mut s = String::new();
                while i < chars.len() {
                    if chars[i] == '"' {
                        if i + 1 < chars.len() && chars[i + 1] == '"' {
                            s.push('"');
                            i += 2;
                            continue;
                        }
                        i += 1;
                        break;
                    }
                    s.push(chars[i]);
                    i += 1;
                }
                out.push(s);
                while i < chars.len() && chars[i] != ',' {
                    i += 1;
                }
                i += 1;
            } else {
                let start = i;
                while i < chars.len() && chars[i] != ',' {
                    i += 1;
                }
                let raw: String = chars[start..i].iter().collect();
                let t = raw.trim();
                out.push(match t {
                    "#TRUE#" => "1".into(),
                    "#FALSE#" => "0".into(),
                    "#NULL#" => String::new(),
                    _ => t.to_string(),
                });
                i += 1;
            }
        }
    }
    out
}

struct Reader {
    tokens: Vec<String>,
    pos: usize,
}

impl Reader {
    fn next(&mut self) -> Result<String, String> {
        let t = self.tokens.get(self.pos).cloned().ok_or("O arquivo terminou antes do esperado.")?;
        self.pos += 1;
        Ok(t)
    }

    fn int(&mut self) -> Result<usize, String> {
        let t = self.next()?;
        t.trim()
            .replace(',', ".")
            .parse::<f64>()
            .ok()
            .filter(|v| *v >= 0.0)
            .map(|v| v as usize)
            .ok_or_else(|| format!("Esperava um número inteiro e encontrei \"{}\".", t))
    }

    fn peek(&self) -> Option<&str> {
        self.tokens.get(self.pos).map(|s| s.as_str())
    }

    fn rest(&self) -> Vec<String> {
        self.tokens[self.pos.min(self.tokens.len())..].to_vec()
    }
}

fn value(cell: &str) -> String {
    let t = cell.trim();
    let lower = t.to_lowercase();
    match lower.as_str() {
        "infinity" | "+infinity" | "inf" => "∞".into(),
        "-infinity" | "-inf" => "-∞".into(),
        _ => {
            if let Ok(v) = t.replace(',', ".").parse::<f64>() {
                if v >= crate::mechanics::BOUND_INFINITY {
                    return "∞".into();
                }
                if v <= -crate::mechanics::BOUND_INFINITY {
                    return "-∞".into();
                }
            }
            t.to_string()
        }
    }
}

fn yes(cell: &str) -> bool {
    matches!(cell.trim().to_lowercase().as_str(), "y" | "yes" | "s" | "sim" | "1" | "true")
}

fn relation(cell: &str) -> String {
    let t = cell.trim();
    if t.starts_with('>') {
        ">=".into()
    } else if t.starts_with('=') {
        "=".into()
    } else {
        "<=".into()
    }
}

fn grid(cells: &[String], cols: usize) -> Vec<Vec<String>> {
    cells.chunks(cols).map(|r| r.to_vec()).collect()
}

fn fail(what: &str, found: usize, expected: &[usize]) -> String {
    let list: Vec<String> = expected.iter().map(|e| e.to_string()).collect();
    format!("{}: o arquivo tem {} células, mas o formato do original pede {}.", what, found, list.join(" ou "))
}

pub fn import(text: &str) -> Result<Imported, String> {
    let mut r = Reader { tokens: tokenize(text), pos: 0 };
    let code = r.int()?;
    let module = code / 1000;
    let variant = code % 1000;
    let _width = r.int()?;
    let _decimals = r.int()?;
    let _flag = r.next()?;
    let title = r.next()?;
    match module {
        1 | 2 => algebra(&mut r, module == 2, title),
        3 | 5 => linear(&mut r, module == 5, title),
        4 => transport(&mut r, title),
        6 => queue(&mut r, title),
        7 => network(&mut r, variant, title),
        8 => project(&mut r, variant, title),
        9 => game(&mut r, title),
        _ => Err(format!("Código de módulo desconhecido: {}.", code)),
    }
}

fn algebra(r: &mut Reader, with_rhs: bool, title: String) -> Result<Imported, String> {
    let n = r.int()?;
    let cells = r.rest();
    let cols = if cells.len() == n * (n + 1) {
        n + 1
    } else if cells.len() == n * n {
        n
    } else {
        return Err(fail("Matriz", cells.len(), &[n * n, n * (n + 1)]));
    };
    let rows = grid(&cells, cols);
    let a: Vec<Vec<String>> = rows.iter().map(|row| row[..n].iter().map(|c| value(c)).collect()).collect();
    let b: Vec<String> = rows.iter().map(|row| if cols > n { value(&row[n]) } else { String::new() }).collect();
    Ok(Imported {
        module: if with_rhs { "linear_equations" } else { "matrix_inverse" },
        data: json!({ "title": title, "n": n, "a": a, "b": b }),
    })
}

fn linear(r: &mut Reader, integer: bool, title: String) -> Result<Imported, String> {
    let n = r.int()?;
    let m = r.int()?;
    let mut maximize = true;
    if let Some(p) = r.peek() {
        let low = p.to_lowercase();
        if low.contains("min") || low.contains("max") {
            maximize = !low.contains("min");
            r.pos += 1;
        }
    }
    let cells = r.rest();
    let cols = n + 2;
    let base = m + 5;
    let rows_count = if cells.len() == (base + 1) * cols {
        base + 1
    } else if cells.len() == base * cols {
        base
    } else {
        return Err(fail("Programação linear", cells.len(), &[base * cols, (base + 1) * cols]));
    };
    let g = grid(&cells, cols);
    let var_names: Vec<String> = (0..n)
        .map(|j| {
            let s = g[0][j].trim();
            if s.is_empty() { format!("x{}", j + 1) } else { s.to_string() }
        })
        .collect();
    let obj: Vec<String> = (0..n).map(|j| value(&g[1][j])).collect();
    let mut a = Vec::new();
    let mut rel = Vec::new();
    let mut rhs = Vec::new();
    for i in 0..m {
        let row = &g[2 + i];
        a.push((0..n).map(|j| value(&row[j])).collect::<Vec<_>>());
        rel.push(relation(&row[n]));
        rhs.push(value(&row[n + 1]));
    }
    let lower: Vec<String> = (0..n).map(|j| value(&g[m + 2][j])).collect();
    let upper: Vec<String> = (0..n)
        .map(|j| {
            let v = value(&g[m + 3][j]);
            if v.is_empty() { "∞".into() } else { v }
        })
        .collect();
    let unrestricted: Vec<bool> = (0..n).map(|j| yes(&g[m + 4][j])).collect();
    let integer_flags: Vec<bool> = if rows_count > base {
        (0..n).map(|j| yes(&g[m + 5][j])).collect()
    } else {
        vec![integer; n]
    };
    let con_names: Vec<String> = (0..m).map(|i| format!("Restr {}", i + 1)).collect();
    Ok(Imported {
        module: if integer { "integer_programming" } else { "linear_programming" },
        data: json!({
            "title": title,
            "maximize": maximize,
            "nvars": n,
            "ncons": m,
            "var_names": var_names,
            "obj": obj,
            "con_names": con_names,
            "a": a,
            "rel": rel,
            "rhs": rhs,
            "lower": lower,
            "upper": upper,
            "unrestricted": unrestricted,
            "integer": integer_flags,
        }),
    })
}

fn transport(r: &mut Reader, title: String) -> Result<Imported, String> {
    let ns = r.int()?;
    let nd = r.int()?;
    let cells = r.rest();
    let layouts = [(ns + 2, nd + 2, true, true), (ns + 2, nd + 1, true, false), (ns + 1, nd + 1, false, false)];
    let Some(&(rows, cols, name_row, name_col)) = layouts.iter().find(|(a, b, _, _)| a * b == cells.len()) else {
        return Err(fail("Transporte", cells.len(), &layouts.iter().map(|(a, b, _, _)| a * b).collect::<Vec<_>>()));
    };
    let g = grid(&cells, cols);
    let r0 = usize::from(name_row);
    let c0 = usize::from(name_col);
    let sources: Vec<String> = (0..ns)
        .map(|i| {
            let s = if name_col { g[r0 + i][0].trim().to_string() } else { String::new() };
            if s.is_empty() { format!("S{}", i + 1) } else { s }
        })
        .collect();
    let dests: Vec<String> = (0..nd)
        .map(|j| {
            let s = if name_row { g[0][c0 + j].trim().to_string() } else { String::new() };
            if s.is_empty() { format!("D{}", j + 1) } else { s }
        })
        .collect();
    let cost: Vec<Vec<String>> = (0..ns).map(|i| (0..nd).map(|j| value(&g[r0 + i][c0 + j])).collect()).collect();
    let supply: Vec<String> = (0..ns).map(|i| value(&g[r0 + i][cols - 1])).collect();
    let demand: Vec<String> = (0..nd).map(|j| value(&g[rows - 1][c0 + j])).collect();
    Ok(Imported {
        module: "transportation",
        data: json!({
            "title": title,
            "ns": ns,
            "nd": nd,
            "sources": sources,
            "dests": dests,
            "cost": cost,
            "supply": supply,
            "demand": demand,
        }),
    })
}

fn queue_limit(cell: &str) -> String {
    let v = value(cell);
    match v.replace(',', ".").parse::<f64>() {
        Ok(x) if x == crate::mechanics::QUEUE_INFINITY => "∞".into(),
        _ if v.is_empty() => "∞".into(),
        _ => v,
    }
}

fn queue(r: &mut Reader, title: String) -> Result<Imported, String> {
    let k = r.int()?;
    let cells = r.rest();
    let (cols, named) = if cells.len() == k * 6 {
        (6, true)
    } else if cells.len() == k * 5 {
        (5, false)
    } else {
        return Err(fail("Filas", cells.len(), &[k * 5, k * 6]));
    };
    let g = grid(&cells, cols);
    let rows: Vec<Vec<String>> = g
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let (name, d) = if named { (row[0].trim().to_string(), &row[1..]) } else { (String::new(), &row[..]) };
            vec![
                if name.is_empty() { format!("{}", i + 1) } else { name },
                value(&d[0]),
                value(&d[1]),
                value(&d[2]),
                queue_limit(&d[3]),
                queue_limit(&d[4]),
            ]
        })
        .collect();
    Ok(Imported { module: "queuing", data: json!({ "title": title, "rows": rows }) })
}

fn network(r: &mut Reader, variant: usize, title: String) -> Result<Imported, String> {
    let n = r.int()?;
    let mut cells = r.rest();
    let module = match variant {
        1 => "minimal_spanning_tree",
        4 => "maximal_flow",
        _ => "shortest_route",
    };
    let fits = |len: usize| -> Option<(usize, bool)> {
        if len == (n + 1) * n {
            Some((n, false))
        } else if len == (n + 1) * (n + 1) {
            Some((n + 1, true))
        } else {
            None
        }
    };
    let layout = match fits(cells.len()) {
        Some(l) => l,
        None => {
            cells.pop();
            fits(cells.len()).ok_or_else(|| fail("Redes", cells.len() + 1, &[(n + 1) * n + 1, (n + 1) * (n + 1) + 1]))?
        }
    };
    let (cols, name_col) = layout;
    let g = grid(&cells, cols);
    let c0 = usize::from(name_col);
    let names: Vec<String> = (0..n)
        .map(|j| {
            let s = g[0][c0 + j].trim().to_string();
            if s.is_empty() { format!("N{}", j + 1) } else { s }
        })
        .collect();
    let matrix: Vec<Vec<String>> = (0..n).map(|i| (0..n).map(|j| value(&g[1 + i][c0 + j])).collect()).collect();
    Ok(Imported { module, data: json!({ "title": title, "n": n, "names": names, "matrix": matrix }) })
}

fn project(r: &mut Reader, variant: usize, title: String) -> Result<Imported, String> {
    let _nodes = r.int()?;
    let pert = variant == 2;
    let width = if pert { 6 } else { 4 };
    let mut cells = r.rest();
    if let Some(first) = cells.first() {
        if first.trim().parse::<usize>().is_ok() && cells.len() % width == 1 {
            cells.remove(0);
        }
    }
    if cells.len() % width != 0 {
        return Err(format!("Projeto: o arquivo tem {} células, que não formam linhas de {} colunas.", cells.len(), width));
    }
    let rows: Vec<Vec<String>> = grid(&cells, width)
        .into_iter()
        .filter(|row| row.iter().any(|c| !c.trim().is_empty()))
        .map(|row| {
            let mut out = vec![row[2].trim().to_string(), row[0].trim().to_string(), row[1].trim().to_string()];
            out.extend(row[3..].iter().map(|c| value(c)));
            out
        })
        .collect();
    Ok(Imported { module: if pert { "pert" } else { "cpm" }, data: json!({ "title": title, "rows": rows }) })
}

fn game(r: &mut Reader, title: String) -> Result<Imported, String> {
    let m = r.int()?;
    let n = r.int()?;
    let cells = r.rest();
    if cells.len() != m * n {
        return Err(fail("Jogos", cells.len(), &[m * n]));
    }
    let payoff: Vec<Vec<String>> = grid(&cells, n).into_iter().map(|row| row.iter().map(|c| value(c)).collect()).collect();
    Ok(Imported { module: "zero_sum_game", data: json!({ "title": title, "m": m, "n": n, "payoff": payoff }) })
}

fn quoted(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "'"))
}

fn plain(cell: &str) -> String {
    match cell.trim() {
        "∞" | "inf" | "infinito" => "infinity".into(),
        "-∞" | "-inf" | "-infinito" => "-infinity".into(),
        t => t.to_string(),
    }
}

fn strings(v: &Value, key: &str) -> Vec<String> {
    v.get(key)
        .and_then(|x| x.as_array())
        .map(|a| a.iter().map(|s| s.as_str().unwrap_or("").to_string()).collect())
        .unwrap_or_default()
}

fn bools(v: &Value, key: &str) -> Vec<bool> {
    v.get(key).and_then(|x| x.as_array()).map(|a| a.iter().map(|b| b.as_bool().unwrap_or(false)).collect()).unwrap_or_default()
}

fn matrix(v: &Value, key: &str) -> Vec<Vec<String>> {
    v.get(key)
        .and_then(|x| x.as_array())
        .map(|rows| {
            rows.iter()
                .map(|r| r.as_array().map(|c| c.iter().map(|s| s.as_str().unwrap_or("").to_string()).collect()).unwrap_or_default())
                .collect()
        })
        .unwrap_or_default()
}

fn num(v: &Value, key: &str) -> usize {
    v.get(key).and_then(|x| x.as_u64()).unwrap_or(0) as usize
}

pub fn export(module: &str, data: &Value) -> Result<String, String> {
    let title = data.get("title").and_then(|t| t.as_str()).unwrap_or("").to_string();
    let (code, header, cells): (usize, Vec<String>, Vec<String>) = match module {
        "matrix_inverse" | "linear_equations" => {
            let n = num(data, "n");
            let a = matrix(data, "a");
            let b = strings(data, "b");
            let rhs = module == "linear_equations";
            let mut cells = Vec::new();
            for i in 0..n {
                for j in 0..n {
                    cells.push(plain(a.get(i).and_then(|r| r.get(j)).map(|s| s.as_str()).unwrap_or("")));
                }
                if rhs {
                    cells.push(plain(b.get(i).map(|s| s.as_str()).unwrap_or("")));
                }
            }
            (if rhs { 2000 } else { 1000 }, vec![n.to_string()], cells)
        }
        "linear_programming" | "integer_programming" => {
            let integer = module == "integer_programming";
            let n = num(data, "nvars");
            let m = num(data, "ncons");
            let maximize = data.get("maximize").and_then(|b| b.as_bool()).unwrap_or(true);
            let names = strings(data, "var_names");
            let obj = strings(data, "obj");
            let a = matrix(data, "a");
            let rel = strings(data, "rel");
            let rhs = strings(data, "rhs");
            let lower = strings(data, "lower");
            let upper = strings(data, "upper");
            let unr = bools(data, "unrestricted");
            let int = bools(data, "integer");
            let get = |v: &Vec<String>, j: usize| v.get(j).cloned().unwrap_or_default();
            let mut cells = Vec::new();
            let mut row = |vals: Vec<String>, tail: [String; 2]| {
                cells.extend(vals);
                cells.extend(tail);
            };
            row((0..n).map(|j| get(&names, j)).collect(), [String::new(), String::new()]);
            row((0..n).map(|j| plain(&get(&obj, j))).collect(), [String::new(), String::new()]);
            for i in 0..m {
                let r = a.get(i).cloned().unwrap_or_default();
                let rl = match get(&rel, i).trim() {
                    s if s.starts_with('>') => ">".to_string(),
                    s if s.starts_with('=') => "=".to_string(),
                    _ => "<".to_string(),
                };
                row((0..n).map(|j| plain(&get(&r, j))).collect(), [rl, plain(&get(&rhs, i))]);
            }
            row((0..n).map(|j| plain(&get(&lower, j))).collect(), [String::new(), String::new()]);
            row((0..n).map(|j| plain(&get(&upper, j))).collect(), [String::new(), String::new()]);
            row((0..n).map(|j| if unr.get(j).copied().unwrap_or(false) { "y".into() } else { "n".into() }).collect(), [String::new(), String::new()]);
            if integer {
                row((0..n).map(|j| if int.get(j).copied().unwrap_or(false) { "y".into() } else { "n".into() }).collect(), [String::new(), String::new()]);
            }
            let sense = if maximize { "Maximize" } else { "Minimize" };
            (if integer { 5000 } else { 3000 }, vec![n.to_string(), m.to_string(), quoted(sense)], cells)
        }
        "transportation" => {
            let ns = num(data, "ns");
            let nd = num(data, "nd");
            let sources = strings(data, "sources");
            let dests = strings(data, "dests");
            let cost = matrix(data, "cost");
            let supply = strings(data, "supply");
            let demand = strings(data, "demand");
            let mut cells = vec![String::new()];
            cells.extend((0..nd).map(|j| dests.get(j).cloned().unwrap_or_default()));
            cells.push(String::new());
            for i in 0..ns {
                cells.push(sources.get(i).cloned().unwrap_or_default());
                cells.extend((0..nd).map(|j| plain(cost.get(i).and_then(|r| r.get(j)).map(|s| s.as_str()).unwrap_or(""))));
                cells.push(plain(supply.get(i).map(|s| s.as_str()).unwrap_or("")));
            }
            cells.push(String::new());
            cells.extend((0..nd).map(|j| plain(demand.get(j).map(|s| s.as_str()).unwrap_or(""))));
            cells.push(String::new());
            (4000, vec![ns.to_string(), nd.to_string()], cells)
        }
        "queuing" => {
            let rows = matrix(data, "rows");
            let mut cells = Vec::new();
            for r in &rows {
                for (k, c) in r.iter().enumerate().skip(1).take(5) {
                    let v = if k >= 4 && matches!(c.trim(), "" | "∞" | "inf" | "infinito") { "9999".to_string() } else { plain(c) };
                    cells.push(v);
                }
            }
            (6000, vec![rows.len().to_string()], cells)
        }
        "minimal_spanning_tree" | "shortest_route" | "maximal_flow" => {
            let n = num(data, "n");
            let names = strings(data, "names");
            let mtx = matrix(data, "matrix");
            let mut cells: Vec<String> = (0..n).map(|j| names.get(j).cloned().unwrap_or_default()).collect();
            for i in 0..n {
                cells.extend((0..n).map(|j| plain(mtx.get(i).and_then(|r| r.get(j)).map(|s| s.as_str()).unwrap_or(""))));
            }
            cells.push(String::new());
            let code = match module {
                "minimal_spanning_tree" => 7001,
                "maximal_flow" => 7004,
                _ => 7002,
            };
            (code, vec![n.to_string()], cells)
        }
        "cpm" | "pert" => {
            let rows = matrix(data, "rows");
            let width = if module == "pert" { 6 } else { 4 };
            let mut nodes = 0usize;
            let mut cells = Vec::new();
            for r in &rows {
                let g = |k: usize| r.get(k).cloned().unwrap_or_default();
                for k in [1, 2] {
                    nodes = nodes.max(g(k).trim().parse::<usize>().unwrap_or(0));
                }
                cells.push(g(1));
                cells.push(g(2));
                cells.push(g(0));
                for k in 3..width {
                    cells.push(plain(&g(k)));
                }
            }
            (if module == "pert" { 8002 } else { 8001 }, vec![nodes.to_string(), rows.len().to_string()], cells)
        }
        "zero_sum_game" => {
            let m = num(data, "m");
            let n = num(data, "n");
            let p = matrix(data, "payoff");
            let mut cells = Vec::new();
            for i in 0..m {
                cells.extend((0..n).map(|j| plain(p.get(i).and_then(|r| r.get(j)).map(|s| s.as_str()).unwrap_or(""))));
            }
            (9000, vec![m.to_string(), n.to_string()], cells)
        }
        _ => return Err("Este módulo não tem formato no programa original.".into()),
    };
    let mut out = vec![
        code.to_string(),
        crate::mechanics::OUTPUT_WIDTH.to_string(),
        crate::mechanics::OUTPUT_DECIMALS.to_string(),
        "1".to_string(),
        quoted(&title),
    ];
    out.extend(header);
    out.extend(cells.iter().map(|c| quoted(c)));
    Ok(out.join("\r\n") + "\r\n")
}
