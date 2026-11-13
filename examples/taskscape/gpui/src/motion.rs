//! Motion (DESIGN.md, "Motion"): the durations and easings from tokens.css, and the clock every
//! animation in the app reads.
//!
//! GPUI redraws a window when asked (`request_animation_frame`), and each animated surface works
//! out how far it has got from this clock. The clock is the real one, except when something
//! outside the app steps it frame by frame (a recording of a clip), so that an animation is drawn
//! at exact 16 ms steps whatever the machine's load.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use gpui::{SharedString, Window};

static START: OnceLock<Instant> = OnceLock::new();
static MANUAL: AtomicBool = AtomicBool::new(false);
static MANUAL_MS: AtomicU64 = AtomicU64::new(0);
static REDUCED: AtomicBool = AtomicBool::new(false);

/// Time since the app started, on the animation clock.
pub fn now() -> Duration {
    if MANUAL.load(Ordering::SeqCst) {
        Duration::from_micros(MANUAL_MS.load(Ordering::SeqCst))
    } else {
        START.get_or_init(Instant::now).elapsed()
    }
}

/// Holds the clock at its current time; `step` then moves it on by hand.
pub fn hold() {
    let t = now();
    MANUAL_MS.store(t.as_micros() as u64, Ordering::SeqCst);
    MANUAL.store(true, Ordering::SeqCst);
}

pub fn step(by: Duration) {
    MANUAL_MS.fetch_add(by.as_micros() as u64, Ordering::SeqCst);
}

/// Lets the clock run again from where it was held (it never goes back).
pub fn release() {
    if !MANUAL.load(Ordering::SeqCst) {
        return;
    }
    let held = Duration::from_micros(MANUAL_MS.load(Ordering::SeqCst));
    let real = START.get_or_init(Instant::now).elapsed();
    if held > real {
        // The held clock ran ahead of the real one: wait the difference out in the real clock's
        // place, so that time keeps going forward for animations that are running.
        std::thread::sleep(held - real);
    }
    MANUAL.store(false, Ordering::SeqCst);
}

/// Reduced motion (the app's setting, or the system's): every animation ends at once.
pub fn set_reduced(on: bool) {
    REDUCED.store(on, Ordering::SeqCst);
}
pub fn reduced() -> bool {
    REDUCED.load(Ordering::SeqCst)
}

/// A CSS `cubic-bezier(x1, y1, x2, y2)`.
#[derive(Clone, Copy)]
pub struct Ease(pub f32, pub f32, pub f32, pub f32);

/// `--ease-standard`, `cubic-bezier(0.2, 0, 0, 1)`.
pub const STANDARD: Ease = Ease(0.2, 0.0, 0.0, 1.0);
/// `--ease-out`, `cubic-bezier(0.22, 1, 0.36, 1)`.
pub const OUT: Ease = Ease(0.22, 1.0, 0.36, 1.0);

impl Ease {
    pub fn at(self, t: f32) -> f32 {
        if t <= 0.0 {
            return 0.0;
        }
        if t >= 1.0 {
            return 1.0;
        }
        let Ease(x1, y1, x2, y2) = self;
        let bez = |a: f32, b: f32, s: f32| {
            let u = 1.0 - s;
            3.0 * u * u * s * a + 3.0 * u * s * s * b + s * s * s
        };
        // Solve x(s) = t by bisection; then y(s).
        let (mut lo, mut hi) = (0.0f32, 1.0f32);
        for _ in 0..40 {
            let mid = (lo + hi) / 2.0;
            if bez(x1, x2, mid) < t {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        bez(y1, y2, (lo + hi) / 2.0)
    }
}

/// The named motions of DESIGN.md.
#[derive(Clone, Copy)]
pub struct Motion {
    pub ms: u64,
    pub ease: Ease,
}
/// rise: 110 ms, up 3 px and in from transparent.
pub const RISE: Motion = Motion { ms: 110, ease: STANDARD };
/// panel-in: 120 ms, in from 6 px right.
pub const PANEL_IN: Motion = Motion { ms: 120, ease: STANDARD };
/// sheet-in: 140 ms, scale 0.96 → 1 and in from transparent.
pub const SHEET_IN: Motion = Motion { ms: 140, ease: STANDARD };
/// The scrim behind a sheet or the palette fades up a little slower than what it dims
/// (tokens.css `.animate-scrim-in`: 160 ms).
pub const SCRIM_IN: Motion = Motion { ms: 160, ease: STANDARD };
/// The capture bar's notes panel growing open.
pub const NOTES_OPEN: Motion = Motion { ms: 180, ease: STANDARD };
/// The done check's tick: 160 ms after 80 ms.
pub const CHECK_DRAW: Motion = Motion { ms: 160, ease: STANDARD };

/// The animations a surface is running, each keyed by what it shows (an inspector rises again
/// when it shows another task).
#[derive(Default)]
pub struct Anims {
    running: HashMap<&'static str, (SharedString, Duration)>,
}

impl Anims {
    /// How far (0…1, eased) the animation in `slot` has got for `key`; starts it when the key
    /// changes. Asks the window for another frame until it ends.
    pub fn progress(&mut self, slot: &'static str, key: impl Into<SharedString>, m: Motion, window: &mut Window) -> f32 {
        self.progress_after(slot, key, m, 0, window)
    }

    pub fn progress_after(&mut self, slot: &'static str, key: impl Into<SharedString>, m: Motion, delay_ms: u64, window: &mut Window) -> f32 {
        let key = key.into();
        let now = now();
        let entry = self.running.entry(slot).or_insert_with(|| (key.clone(), now));
        if entry.0 != key {
            *entry = (key, now);
        }
        if reduced() {
            return 1.0;
        }
        let elapsed = now.saturating_sub(entry.1).as_secs_f32() * 1000.0 - delay_ms as f32;
        let t = (elapsed / m.ms as f32).clamp(0.0, 1.0);
        if t < 1.0 {
            window.request_animation_frame();
            ANIMATING.store(true, Ordering::SeqCst);
        }
        m.ease.at(t)
    }

    /// Forgets the animation in `slot`, so that it plays again next time it is shown.
    pub fn reset(&mut self, slot: &'static str) {
        self.running.remove(slot);
    }
}

static ANIMATING: AtomicBool = AtomicBool::new(false);

/// Whether any animation asked for another frame since the last call (the adapter's settle).
pub fn take_animating() -> bool {
    ANIMATING.swap(false, Ordering::SeqCst)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn easing_ends() {
        assert_eq!(STANDARD.at(0.0), 0.0);
        assert_eq!(STANDARD.at(1.0), 1.0);
        let mid = STANDARD.at(0.5);
        assert!(mid > 0.7 && mid < 0.95, "{mid}");
    }
}
