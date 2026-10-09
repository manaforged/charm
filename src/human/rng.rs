use std::time::{Duration, SystemTime, UNIX_EPOCH};

const GOLDEN: u64 = 0x9E37_79B9_7F4A_7C15;
const MIX_A: u64 = 0xBF58_476D_1CE4_E5B9;
const MIX_B: u64 = 0x94D0_49BB_1331_11EB;
const MANTISSA_SHIFT: u32 = 11;
const MANTISSA_SCALE: f64 = 1.0 / (1u64 << 53) as f64;
const GAUSS_FLOOR: f64 = 1e-10;
pub(crate) const MS_PER_SEC: f64 = 1000.0;

#[derive(Debug, Clone)]
pub struct Rng {
    state: u64,
}

impl Rng {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    #[must_use]
    pub fn from_entropy() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(GOLDEN);
        let local = 0u8;
        let addr = &local as *const u8 as u64;
        Self::new(nanos ^ addr.rotate_left(32))
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(GOLDEN);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(MIX_A);
        z = (z ^ (z >> 27)).wrapping_mul(MIX_B);
        z ^ (z >> 31)
    }

    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> MANTISSA_SHIFT) as f64 * MANTISSA_SCALE
    }

    pub fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + self.next_f64() * (hi - lo)
    }

    pub fn range_int(&mut self, lo: u64, hi: u64) -> u64 {
        if hi <= lo {
            return lo;
        }
        lo + self.next_u64() % (hi - lo + 1)
    }

    pub fn chance(&mut self, p: f64) -> bool {
        self.next_f64() < p
    }

    pub fn gauss(&mut self) -> f64 {
        let u = self.next_f64().max(GAUSS_FLOOR);
        let v = self.next_f64();
        (-2.0 * u.ln()).sqrt() * (std::f64::consts::TAU * v).cos()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Band {
    pub lo: f64,
    pub hi: f64,
}

impl Band {
    pub(super) fn sample(self, rng: &mut Rng) -> f64 {
        rng.range(self.lo, self.hi)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DelayBand {
    pub lo: Duration,
    pub hi: Duration,
}

impl DelayBand {
    pub(super) fn sample_ms(self, rng: &mut Rng) -> f64 {
        rng.range(
            self.lo.as_secs_f64() * MS_PER_SEC,
            self.hi.as_secs_f64() * MS_PER_SEC,
        )
    }

    pub fn pick(self, rng: &mut Rng) -> Duration {
        Duration::from_millis(rng.range_int(self.lo.as_millis() as u64, self.hi.as_millis() as u64))
    }
}

pub(super) fn millis(lo: u64, hi: u64) -> DelayBand {
    DelayBand {
        lo: Duration::from_millis(lo),
        hi: Duration::from_millis(hi),
    }
}

pub(crate) fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * MS_PER_SEC
}

pub(crate) fn from_ms(v: f64) -> Duration {
    Duration::from_secs_f64(v.max(0.0) / MS_PER_SEC)
}
