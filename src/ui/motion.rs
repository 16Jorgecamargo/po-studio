use eframe::egui::{self, Id, Pos2, Rect};
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

static REDUCED: AtomicBool = AtomicBool::new(false);
static EPOCH: AtomicU64 = AtomicU64::new(0);

pub fn set_reduced(v: bool) {
    REDUCED.store(v, Ordering::Relaxed);
}

pub fn reduced() -> bool {
    REDUCED.load(Ordering::Relaxed)
}

pub fn bump() {
    EPOCH.fetch_add(1, Ordering::Relaxed);
}

pub fn ease_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

pub fn ease_in_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

pub fn ease_out_back(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let c1 = 1.70158;
    let c3 = c1 + 1.0;
    1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
}

pub fn phase(elapsed: f32, delay: f32, duration: f32) -> f32 {
    ((elapsed - delay) / duration).clamp(0.0, 1.0)
}

pub fn clock(ctx: &egui::Context, key: impl Hash + Debug, horizon: f32) -> f32 {
    if reduced() {
        return f32::INFINITY;
    }
    let id = Id::unique(("motion_clock", key));
    let now = ctx.input(|i| i.time);
    let epoch = EPOCH.load(Ordering::Relaxed);
    let start = ctx.data_mut(|d| match d.get_temp::<(u64, f64)>(id) {
        Some((e, s)) if e == epoch => s,
        _ => {
            d.insert_temp(id, (epoch, now));
            now
        }
    });
    let elapsed = (now - start) as f32;
    if elapsed < horizon {
        ctx.request_repaint();
    }
    elapsed
}

pub fn fade(ctx: &egui::Context, key: impl Hash + Debug, duration: f32) -> f32 {
    ease_out_cubic(phase(clock(ctx, key, duration), 0.0, duration))
}

#[derive(Clone, Copy)]
struct Slide {
    from: Rect,
    to: Rect,
    start: f64,
    duration: f32,
}

impl Slide {
    fn at(&self, now: f64) -> (Rect, bool) {
        let t = if reduced() || self.duration <= 0.0 {
            1.0
        } else {
            (((now - self.start) as f32) / self.duration).clamp(0.0, 1.0)
        };
        let e = ease_out_back_soft(t);
        let lerp = |a: Pos2, b: Pos2| a + (b - a) * e;
        (Rect::from_min_max(lerp(self.from.min, self.to.min), lerp(self.from.max, self.to.max)), t >= 1.0)
    }
}

fn ease_out_back_soft(t: f32) -> f32 {
    let c1 = 1.2;
    let c3 = c1 + 1.0;
    1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
}

pub fn slide_peek(ctx: &egui::Context, id: Id) -> Option<Rect> {
    let now = ctx.input(|i| i.time);
    ctx.data(|d| d.get_temp::<Slide>(id)).map(|s| s.at(now).0)
}

pub fn slide_clear(ctx: &egui::Context, id: Id) {
    ctx.data_mut(|d| d.remove::<Slide>(id));
}

pub fn slide_to(ctx: &egui::Context, id: Id, target: Rect, duration: f32) -> Rect {
    let now = ctx.input(|i| i.time);
    let stored = ctx.data(|d| d.get_temp::<Slide>(id));
    let slide = match stored {
        None => Slide { from: target, to: target, start: now - duration as f64, duration },
        Some(s) => {
            let moved = (s.to.min - target.min).length() > 0.5 || (s.to.max - target.max).length() > 0.5;
            if moved {
                Slide { from: s.at(now).0, to: target, start: now, duration }
            } else {
                s
            }
        }
    };
    ctx.data_mut(|d| d.insert_temp(id, slide));
    let (rect, done) = slide.at(now);
    if !done {
        ctx.request_repaint();
    }
    rect
}
