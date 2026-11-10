use serde::Serialize;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CellStyle {
    Normal,
    Head,
    Green,
    Red,
    Pivot,
    Bold,
    Muted,
    RedText,
}

#[derive(Clone, Debug, Serialize)]
pub struct Cell {
    pub text: String,
    pub style: CellStyle,
}

impl Cell {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: CellStyle::Normal,
        }
    }
    pub fn styled(text: impl Into<String>, style: CellStyle) -> Self {
        Self {
            text: text.into(),
            style,
        }
    }
}

pub fn c(text: impl Into<String>) -> Cell {
    Cell::new(text)
}

pub fn cs(text: impl Into<String>, style: CellStyle) -> Cell {
    Cell::styled(text, style)
}

pub fn n(v: f64) -> Cell {
    Cell::new(crate::format::num(v))
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Section {
    pub title: String,
    pub subtitle: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<Cell>>,
    pub notes: Vec<String>,
    pub lines: Vec<String>,
}

impl Section {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Default::default()
        }
    }
    pub fn sub(mut self, s: impl Into<String>) -> Self {
        self.subtitle = s.into();
        self
    }
    pub fn headers(mut self, h: &[&str]) -> Self {
        self.headers = h.iter().map(|s| s.to_string()).collect();
        self
    }
    pub fn headers_owned(mut self, h: Vec<String>) -> Self {
        self.headers = h;
        self
    }
    pub fn row(&mut self, r: Vec<Cell>) {
        self.rows.push(r);
    }
    pub fn note(&mut self, s: impl Into<String>) {
        self.notes.push(s.into());
    }
    pub fn line(&mut self, s: impl Into<String>) {
        self.lines.push(s.into());
    }
}

pub fn sections_to_text(title: &str, sections: &[Section]) -> String {
    let mut out = String::new();
    out.push_str(title);
    out.push('\n');
    out.push_str(&"=".repeat(title.chars().count().max(10)));
    out.push_str("\n\n");
    for s in sections {
        if !s.title.is_empty() {
            out.push_str(&s.title);
            out.push('\n');
        }
        if !s.subtitle.is_empty() {
            out.push_str(&s.subtitle);
            out.push('\n');
        }
        for l in &s.lines {
            out.push_str(l);
            out.push('\n');
        }
        let ncol = s.headers.len().max(s.rows.iter().map(|r| r.len()).max().unwrap_or(0));
        if ncol > 0 {
            let mut w = vec![0usize; ncol];
            for (i, h) in s.headers.iter().enumerate() {
                w[i] = w[i].max(h.chars().count());
            }
            for r in &s.rows {
                for (i, cell) in r.iter().enumerate() {
                    w[i] = w[i].max(cell.text.chars().count());
                }
            }
            let fmt_row = |cells: Vec<String>| -> String {
                cells
                    .iter()
                    .enumerate()
                    .map(|(i, t)| format!("{:>width$}", t, width = w[i] + 2))
                    .collect::<String>()
            };
            if !s.headers.is_empty() {
                out.push_str(&fmt_row(s.headers.clone()));
                out.push('\n');
            }
            for r in &s.rows {
                out.push_str(&fmt_row(r.iter().map(|c| c.text.clone()).collect()));
                out.push('\n');
            }
        }
        for nt in &s.notes {
            out.push_str(nt);
            out.push('\n');
        }
        out.push('\n');
    }
    out
}

pub fn title_or_untitled(title: &str) -> String {
    if title.is_empty() {
        "Sem título".to_string()
    } else {
        title.to_string()
    }
}
