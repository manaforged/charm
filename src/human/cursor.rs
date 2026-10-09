use std::f64::consts::{PI, TAU};
use std::time::Duration;

use super::rng::{Band, DelayBand, MS_PER_SEC, Rng, from_ms, millis, ms};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Target {
    Point(Point),
    Rect(Rect),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Step {
    pub point: Point,
    pub after: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CursorOptions {
    pub fitts_a_ms: f64,
    pub fitts_b_ms: f64,
    pub default_target_width: f64,
    pub min_target_width: f64,
    pub min_fitts_distance: f64,
    pub tremor_hz: Vec<f64>,
    pub hurry_rate: f64,
    pub hurry_factor: Band,
    pub normal_factor: Band,
    pub base_submovements: usize,
    pub long_move_distance: f64,
    pub extra_submovement_rate: f64,
    pub max_submovements: usize,
    pub primary_duration_share: Band,
    pub min_primary_duration: Duration,
    pub primary_share: Band,
    pub corrective_share: Band,
    pub share_skew_sd: f64,
    pub overlap: Band,
    pub corrective_duration: DelayBand,
    pub min_distance: f64,
    pub bow: Band,
    pub bow_flip_rate: f64,
    pub arc_skew: Band,
    pub tremor_base: f64,
    pub tremor_per_px: f64,
    pub tremor_max: f64,
    pub tremor_axial_scale: f64,
    pub stall_rate: f64,
    pub stall_at: Band,
    pub stall_delay: DelayBand,
    pub sample_interval: DelayBand,
    pub min_emit_px: f64,
    pub settle_rate: f64,
    pub settle_nudges_min: u64,
    pub settle_nudges_max: u64,
    pub settle_delay: DelayBand,
    pub settle_sd_px: f64,
    pub settle_return_delay: DelayBand,
    pub move_delay: DelayBand,
    pub hesitate: DelayBand,
    pub wait_for_click: DelayBand,
    pub padding_percentage: f64,
    pub max_padding_percentage: f64,
    pub spread: f64,
}

impl Default for CursorOptions {
    fn default() -> Self {
        Self {
            fitts_a_ms: 48.0,
            fitts_b_ms: 108.0,
            default_target_width: 22.0,
            min_target_width: 4.0,
            min_fitts_distance: 1.0,
            tremor_hz: vec![8.5, 11.0, 13.5],
            hurry_rate: 0.14,
            hurry_factor: Band { lo: 0.4, hi: 0.62 },
            normal_factor: Band { lo: 0.8, hi: 1.3 },
            base_submovements: 2,
            long_move_distance: 260.0,
            extra_submovement_rate: 0.34,
            max_submovements: 4,
            primary_duration_share: Band { lo: 0.5, hi: 0.66 },
            min_primary_duration: Duration::from_millis(110),
            primary_share: Band { lo: 0.78, hi: 0.93 },
            corrective_share: Band { lo: 0.55, hi: 0.8 },
            share_skew_sd: 0.04,
            overlap: Band { lo: 0.5, hi: 0.7 },
            corrective_duration: millis(100, 300),
            min_distance: 0.5,
            bow: Band { lo: 0.07, hi: 0.2 },
            bow_flip_rate: 0.5,
            arc_skew: Band { lo: 0.32, hi: 0.66 },
            tremor_base: 0.18,
            tremor_per_px: 0.0008,
            tremor_max: 0.62,
            tremor_axial_scale: 0.6,
            stall_rate: 0.12,
            stall_at: Band { lo: 0.25, hi: 0.75 },
            stall_delay: millis(60, 260),
            sample_interval: millis(7, 11),
            min_emit_px: 1.0,
            settle_rate: 0.3,
            settle_nudges_min: 1,
            settle_nudges_max: 2,
            settle_delay: millis(25, 90),
            settle_sd_px: 1.8,
            settle_return_delay: millis(20, 70),
            move_delay: millis(50, 200),
            hesitate: millis(80, 200),
            wait_for_click: millis(40, 100),
            padding_percentage: 10.0,
            max_padding_percentage: 45.0,
            spread: 0.42,
        }
    }
}

struct Submovement {
    dx: f64,
    dy: f64,
    start: f64,
    duration: f64,
}

fn min_jerk(t: f64) -> f64 {
    let u = t.clamp(0.0, 1.0);
    u * u * u * (10.0 - 15.0 * u + 6.0 * u * u)
}

fn fitts_time(distance: f64, width: f64, o: &CursorOptions, rng: &mut Rng) -> f64 {
    let w = width.max(o.min_target_width);
    let id = (distance.max(o.min_fitts_distance) / w + 1.0).log2();
    let hurry = if rng.chance(o.hurry_rate) {
        o.hurry_factor.sample(rng)
    } else {
        o.normal_factor.sample(rng)
    };
    (o.fitts_a_ms + o.fitts_b_ms * id) * hurry
}

fn plan_submovements(
    dx: f64,
    dy: f64,
    total: f64,
    o: &CursorOptions,
    rng: &mut Rng,
) -> Vec<Submovement> {
    let distance = dx.hypot(dy);
    let long = usize::from(distance > o.long_move_distance);
    let extra = usize::from(rng.chance(o.extra_submovement_rate));
    let count = (o.base_submovements + long + extra).clamp(o.base_submovements, o.max_submovements);
    let mut plan = Vec::with_capacity(count);
    let (mut rx, mut ry, mut start) = (dx, dy, 0.0);
    let mut duration =
        (total * o.primary_duration_share.sample(rng)).max(ms(o.min_primary_duration));
    for i in 0..count {
        let last = i + 1 == count;
        let (sx, sy) = if last {
            (rx, ry)
        } else {
            let band = if i == 0 {
                o.primary_share
            } else {
                o.corrective_share
            };
            let share = band.sample(rng);
            let skew = 1.0 + rng.gauss() * o.share_skew_sd;
            (rx * share * skew, ry * share * skew)
        };
        plan.push(Submovement {
            dx: sx,
            dy: sy,
            start,
            duration,
        });
        rx -= sx;
        ry -= sy;
        if last {
            break;
        }
        start += duration * o.overlap.sample(rng);
        duration = o.corrective_duration.sample_ms(rng);
    }
    plan
}

fn position_at(plan: &[Submovement], from: Point, elapsed: f64) -> Point {
    plan.iter().fold(from, |p, s| {
        let k = min_jerk((elapsed - s.start) / s.duration);
        Point {
            x: p.x + s.dx * k,
            y: p.y + s.dy * k,
        }
    })
}

pub fn target_point(
    rect: Rect,
    padding_percentage: f64,
    options: &CursorOptions,
    rng: &mut Rng,
) -> Point {
    let pad = padding_percentage.clamp(0.0, options.max_padding_percentage) / 100.0;
    let px = rect.width * pad;
    let py = rect.height * pad;
    let mut bias = |lo: f64, hi: f64| {
        let mid = (lo + hi) / 2.0;
        let half = (hi - lo) / 2.0;
        (mid + rng.gauss() * half * options.spread).clamp(lo, hi.max(lo))
    };
    Point {
        x: bias(rect.x + px, rect.x + (rect.width - px).max(px)),
        y: bias(rect.y + py, rect.y + (rect.height - py).max(py)),
    }
}

struct Trace {
    steps: Vec<Step>,
    origin: Point,
}

impl Trace {
    fn wait(&mut self, delay: f64) {
        match self.steps.last_mut() {
            Some(step) => step.after += from_ms(delay),
            None => self.steps.push(Step {
                point: self.origin,
                after: from_ms(delay),
            }),
        }
    }

    fn go(&mut self, point: Point) {
        self.steps.push(Step {
            point,
            after: Duration::ZERO,
        });
    }
}

pub fn path(from: Point, to: Target, options: &CursorOptions, rng: &mut Rng) -> Vec<Step> {
    let o = options;
    let (end, width) = match to {
        Target::Point(p) => (p, o.default_target_width),
        Target::Rect(r) => (
            target_point(r, o.padding_percentage, o, rng),
            r.width.min(r.height).max(o.min_target_width),
        ),
    };
    let mut trace = Trace {
        steps: Vec::new(),
        origin: from,
    };
    let dx = end.x - from.x;
    let dy = end.y - from.y;
    let distance = dx.hypot(dy);
    if distance < o.min_distance {
        trace.go(end);
    } else {
        sweep(&mut trace, from, end, width, o, rng);
        settle(&mut trace, end, o, rng);
    }
    let delay = o.move_delay.pick(rng);
    trace.wait(ms(delay));
    trace.steps
}

fn sweep(trace: &mut Trace, from: Point, end: Point, width: f64, o: &CursorOptions, rng: &mut Rng) {
    let dx = end.x - from.x;
    let dy = end.y - from.y;
    let distance = dx.hypot(dy);
    let total = fitts_time(distance, width, o, rng);
    let plan = plan_submovements(dx, dy, total, o, rng);
    let span = plan
        .iter()
        .fold(0.0_f64, |e, s| e.max(s.start + s.duration));
    let (nx, ny) = (-dy / distance, dx / distance);
    let side = if rng.chance(o.bow_flip_rate) {
        -1.0
    } else {
        1.0
    };
    let bow = o.bow.sample(rng) * distance * side;
    let skew = o.arc_skew.sample(rng);
    let tremor = (o.tremor_base + distance * o.tremor_per_px).min(o.tremor_max);
    let perp_phase: Vec<f64> = o.tremor_hz.iter().map(|_| rng.range(0.0, TAU)).collect();
    let axial_phase: Vec<f64> = o.tremor_hz.iter().map(|_| rng.range(0.0, TAU)).collect();
    let stall = rng.chance(o.stall_rate).then(|| o.stall_at.sample(rng));
    let mut stalled = false;
    let mut elapsed = 0.0;
    let mut emitted = from;
    let exponent = 0.5_f64.ln() / skew.ln();
    while elapsed < span {
        let step = o.sample_interval.sample_ms(rng);
        elapsed = (elapsed + step).min(span);
        if elapsed >= span {
            break;
        }
        let base = position_at(&plan, from, elapsed);
        let u = elapsed / span;
        let arc = (PI * u.powf(exponent)).sin();
        let secs = elapsed / MS_PER_SEC;
        let band = |phase: &[f64]| {
            let v: f64 = o
                .tremor_hz
                .iter()
                .zip(phase)
                .enumerate()
                .map(|(i, (hz, ph))| (secs * hz * TAU + ph).sin() / (i as f64 + 2.0))
                .sum();
            v * tremor * arc
        };
        let perp = bow * arc + band(&perp_phase);
        let axial = band(&axial_phase) * o.tremor_axial_scale;
        let at = Point {
            x: base.x + nx * perp + dx / distance * axial,
            y: base.y + ny * perp + dy / distance * axial,
        };
        if (at.x - emitted.x).hypot(at.y - emitted.y) >= o.min_emit_px {
            trace.go(at);
            emitted = at;
        }
        trace.wait(step);
        if let Some(at_u) = stall
            && !stalled
            && at_u > 0.0
            && u >= at_u
        {
            stalled = true;
            let pause = o.stall_delay.sample_ms(rng);
            trace.wait(pause);
        }
    }
    trace.go(end);
}

fn settle(trace: &mut Trace, end: Point, o: &CursorOptions, rng: &mut Rng) {
    if rng.next_f64() > o.settle_rate {
        return;
    }
    let nudges = rng.range_int(o.settle_nudges_min, o.settle_nudges_max);
    for _ in 0..nudges {
        let pause = o.settle_delay.sample_ms(rng);
        trace.wait(pause);
        let at = Point {
            x: end.x + rng.gauss() * o.settle_sd_px,
            y: end.y + rng.gauss() * o.settle_sd_px,
        };
        trace.go(at);
    }
    let pause = o.settle_return_delay.sample_ms(rng);
    trace.wait(pause);
    trace.go(end);
}
