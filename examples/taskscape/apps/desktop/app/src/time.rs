//! Relative times as the reference writes them (its time.ts): "just now", "12m ago", "3h ago",
//! "yesterday", "3d ago", "2w ago", then the date ("Jul 23").

const CAPTURE_CLOCK: &str = "taskscape.capture-clock";

/// Set the display clock for a headless capture without changing the stored dates.
pub fn set_capture_time(ctx: &egui::Context, epoch: i64) {
    ctx.data_mut(|data| data.insert_temp(egui::Id::unique(CAPTURE_CLOCK), epoch));
}

pub fn now_ms(ctx: &egui::Context) -> i64 {
    ctx.data(|data| data.get_temp(egui::Id::unique(CAPTURE_CLOCK)))
        .unwrap_or_else(taskscape_storage::util::now_millis)
}

/// `ms` and `now` in milliseconds since 1970.
pub fn relative(ms: i64, now: i64) -> String {
    let round = |a: i64, b: i64| (a as f64 / b as f64).round() as i64;
    let sec = round((now - ms).max(0), 1000);
    if sec < 45 {
        return "just now".into();
    }
    let min = round(sec, 60);
    if min < 60 {
        return format!("{min}m ago");
    }
    let hr = round(min, 60);
    if hr < 24 {
        return format!("{hr}h ago");
    }
    let day = round(hr, 24);
    if day == 1 {
        return "yesterday".into();
    }
    if day < 7 {
        return format!("{day}d ago");
    }
    let wk = round(day, 7);
    if wk < 5 {
        return format!("{wk}w ago");
    }
    month_day(ms)
}

/// "Jul 23": the month's short name and the day (in UTC).
pub fn month_day(ms: i64) -> String {
    const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    // Days since 1970-01-01 to a civil date (Howard Hinnant's algorithm).
    let z = ms.div_euclid(86_400_000) + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    format!("{} {day}", MONTHS[(month - 1) as usize])
}

#[cfg(test)]
mod tests {
    use super::*;
    const H: i64 = 3_600_000;

    #[test]
    fn the_reference_steps() {
        let now = 1_784_000_000_000;
        assert_eq!(relative(now - 10_000, now), "just now");
        assert_eq!(relative(now - 12 * 60_000, now), "12m ago");
        assert_eq!(relative(now - 3 * H, now), "3h ago");
        assert_eq!(relative(now - 24 * H, now), "yesterday");
        assert_eq!(relative(now - 3 * 24 * H, now), "3d ago");
        assert_eq!(relative(now - 14 * 24 * H, now), "2w ago");
        // 2026-07-23 12:00 UTC, three months before.
        assert_eq!(relative(1_784_808_000_000, 1_784_808_000_000 + 90 * 24 * H), "Jul 23");
    }
}
