use crate::format::{num, parse_num, parse_or_zero};
use crate::project::{cpm, normal_cdf, pert, Activity, ActivitySchedule, CpmResult, PertActivity};
use crate::report::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct ProjectInput {
    pub title: String,
    pub rows: Vec<Vec<String>>,
}

pub fn cols(pert_mode: bool) -> usize {
    if pert_mode {
        6
    } else {
        4
    }
}

pub fn headers(pert_mode: bool) -> Vec<&'static str> {
    if pert_mode {
        vec!["Atividade", "Nó de", "Nó para", "Otimista a", "Mais provável m", "Pessimista b"]
    } else {
        vec!["Atividade", "Nó de", "Nó para", "Duração"]
    }
}

impl ProjectInput {
    pub fn blank(pert_mode: bool) -> Self {
        Self {
            title: String::new(),
            rows: vec![vec![String::new(); cols(pert_mode)]; 5],
        }
    }

    pub fn fit(&mut self, pert_mode: bool) {
        let k = cols(pert_mode);
        for r in self.rows.iter_mut() {
            r.resize(k, String::new());
        }
    }

    pub fn set_count(&mut self, count: usize, pert_mode: bool) {
        self.rows.resize(count, vec![String::new(); cols(pert_mode)]);
    }

    pub fn example(pert_mode: bool) -> Self {
        let rows: Vec<Vec<&str>> = if pert_mode {
            vec![
                vec!["A", "1", "2", "2", "4", "6"],
                vec!["B", "1", "3", "3", "5", "9"],
                vec!["C", "2", "4", "1", "2", "3"],
                vec!["D", "3", "4", "4", "6", "10"],
                vec!["E", "2", "5", "5", "7", "9"],
                vec!["F", "4", "5", "2", "3", "4"],
                vec!["G", "5", "6", "1", "2", "5"],
            ]
        } else {
            vec![
                vec!["A", "1", "2", "4"],
                vec!["B", "1", "3", "6"],
                vec!["C", "2", "4", "3"],
                vec!["D", "3", "4", "5"],
                vec!["E", "2", "5", "8"],
                vec!["F", "4", "5", "2"],
                vec!["G", "5", "6", "3"],
            ]
        };
        Self {
            title: "Projeto de exemplo".into(),
            rows: rows.iter().map(|r| r.iter().map(|s| s.to_string()).collect()).collect(),
        }
    }

    fn filled_rows(&self) -> Vec<&Vec<String>> {
        self.rows.iter().filter(|r| r.iter().skip(1).any(|c| !c.trim().is_empty())).collect()
    }
}

pub struct ProjectOutcome {
    pub results: Vec<Section>,
    pub iterations: Vec<Section>,
    pub schedule: CpmResult,
    pub path_stats: Option<(f64, f64)>,
}

pub fn solve(input: &ProjectInput, pert_mode: bool) -> Result<ProjectOutcome, String> {
    let rows = input.filled_rows();
    let node = |s: &str| -> usize { parse_num(s).map(|v| v.max(0.0) as usize).unwrap_or(0) };
    let name = |r: &Vec<String>, k: usize| -> String {
        if r[0].trim().is_empty() {
            format!("A{}", k + 1)
        } else {
            r[0].trim().to_string()
        }
    };
    let title = title_or_untitled(&input.title);
    let mut path_stats = None;
    let (mut secs, c) = if pert_mode {
        let acts: Vec<PertActivity> = rows
            .iter()
            .enumerate()
            .map(|(k, r)| PertActivity {
                name: name(r, k),
                from: node(&r[1]),
                to: node(&r[2]),
                a: parse_or_zero(&r[3]),
                m: parse_or_zero(&r[4]),
                b: parse_or_zero(&r[5]),
            })
            .collect();
        let p = pert(&acts)?;
        let mut mv = Section::new("Média e variância das atividades").headers(&["Atividade", "a", "m", "b", "Média", "Variância"]);
        for (i, a) in acts.iter().enumerate() {
            let crit = p.cpm.activities[i].critical;
            let st = if crit { CellStyle::RedText } else { CellStyle::Head };
            mv.row(vec![cs(a.name.clone(), st), n(a.a), n(a.m), n(a.b), cs(num(p.means[i]), CellStyle::Bold), n(p.variances[i])]);
        }
        mv.note("Média = (a + 4m + b) / 6. Variância = ((b − a) / 6)²");
        let mut path = Section::new("Caminho mais longo pelas durações médias").sub(title);
        path.line(format!("Caminho: {}", p.path.iter().map(|&i| acts[i].name.clone()).collect::<Vec<_>>().join(" → ")));
        path.line(format!("Média do caminho = {}. Desvio-padrão do caminho = {}", num(p.path_mean), num(p.path_std)));
        path_stats = Some((p.path_mean, p.path_std));
        (vec![path, mv], p.cpm)
    } else {
        let acts: Vec<Activity> = rows
            .iter()
            .enumerate()
            .map(|(k, r)| Activity {
                name: name(r, k),
                from: node(&r[1]),
                to: node(&r[2]),
                duration: parse_or_zero(&r[3]),
            })
            .collect();
        let c = cpm(&acts)?;
        let mut h = Section::new("Cronograma CPM").sub(title);
        h.line(format!("Duração do projeto = {}", num(c.duration)));
        (vec![h], c)
    };
    let mut sch = Section::new("Cronograma das atividades").headers(&[
        "Atividade",
        "De",
        "Para",
        "Duração",
        "Início cedo",
        "Término cedo",
        "Início tarde",
        "Término tarde",
        "Folga total",
        "Folga livre",
    ]);
    for a in &c.activities {
        let st = if a.critical { CellStyle::RedText } else { CellStyle::Head };
        sch.row(vec![
            cs(a.name.clone(), st),
            crate::report::c(a.from.to_string()),
            crate::report::c(a.to.to_string()),
            n(a.duration),
            n(a.es),
            n(a.ef),
            n(a.ls),
            n(a.lf),
            cs(num(a.total_float), if a.critical { CellStyle::RedText } else { CellStyle::Normal }),
            n(a.free_float),
        ]);
    }
    sch.note("Atividades críticas destacadas em vermelho.");
    if pert_mode {
        secs.insert(1, sch);
    } else {
        secs.push(sch);
    }
    let mut fw = Section::new("Passagem para frente (tempos mais cedo dos nós)");
    for s in &c.forward_steps {
        fw.line(s.clone());
    }
    fw.note("Passagem para frente concluída.");
    let mut bw = Section::new("Passagem para trás (tempos mais tarde dos nós)");
    for s in &c.backward_steps {
        bw.line(s.clone());
    }
    bw.note("Passagem para trás concluída.");
    let mut ev = Section::new("Tempos dos eventos").headers(&["Nó", "Mais cedo", "Mais tarde"]);
    for (i, nd) in c.nodes.iter().enumerate() {
        ev.row(vec![cs(nd.to_string(), CellStyle::Head), n(c.early[i]), n(c.late[i])]);
    }
    Ok(ProjectOutcome {
        results: secs,
        iterations: vec![fw, bw, ev],
        schedule: c,
        path_stats,
    })
}

pub fn delay_message(acts: &[ActivitySchedule], index: usize, delay: f64) -> Option<String> {
    let a = acts.get(index)?;
    if a.critical || delay <= 0.0 {
        return None;
    }
    let start = a.es + delay;
    let mut msg = format!(
        "ATIVIDADE {}: INÍCIO = {}, ATRASO em relação ao início cedo = {}, FOLGA LIVRE = {}.",
        a.name,
        num(start),
        num(delay),
        num(a.free_float)
    );
    if delay > a.free_float + 1e-9 {
        let succ: Vec<String> = acts.iter().filter(|b| b.from == a.to).map(|b| b.name.clone()).collect();
        msg.push_str(&format!(
            " Como o ATRASO excede a FOLGA LIVRE em {}, as atividades seguintes {{{}}} não podem começar antes do tempo {}.",
            num(delay - a.free_float),
            succ.join(", "),
            num(start + a.duration)
        ));
    }
    Some(msg)
}

pub fn deadline_probability(mean: f64, sd: f64, t: f64) -> f64 {
    if sd > 0.0 {
        normal_cdf((t - mean) / sd)
    } else if t >= mean {
        1.0
    } else {
        0.0
    }
}

pub fn report(input: &ProjectInput, pert_mode: bool, results: &[Section], iterations: &[Section]) -> Option<String> {
    if results.is_empty() {
        return None;
    }
    let mut all = results.to_vec();
    all.extend(iterations.iter().cloned());
    let t = if pert_mode { "PLANEJAMENTO DE PROJETOS: PERT" } else { "PLANEJAMENTO DE PROJETOS: CPM" };
    Some(sections_to_text(&format!("{}: {}", t, input.title), &all))
}
