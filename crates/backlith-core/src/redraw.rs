// SPDX-License-Identifier: GPL-3.0-or-later
//! When to redraw the wallpaper: every 5 minutes, or every 15 on battery
//! saver; at once after any change and at midnight; never while the
//! session is locked or Backlith is paused.

use chrono::{Duration, NaiveDateTime, NaiveTime};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RedrawState {
    pub last_redraw: Option<NaiveDateTime>,
    pub dirty: bool,
    pub locked: bool,
    pub battery_saver: bool,
    pub paused: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedrawDecision {
    Now,
    At(NaiveDateTime),
    Hold,
}

pub fn decide_redraw(now: NaiveDateTime, state: &RedrawState) -> RedrawDecision {
    if state.locked || state.paused {
        return RedrawDecision::Hold;
    }
    let Some(last) = state.last_redraw else {
        return RedrawDecision::Now;
    };
    if state.dirty || now.date() != last.date() {
        return RedrawDecision::Now;
    }
    let every = Duration::minutes(if state.battery_saver { 15 } else { 5 });
    let midnight = (last.date() + Duration::days(1)).and_time(NaiveTime::MIN);
    let due = (last + every).min(midnight);
    if now >= due { RedrawDecision::Now } else { RedrawDecision::At(due) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(s: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M").unwrap()
    }
    fn drawn(last: &str) -> RedrawState {
        RedrawState { last_redraw: Some(at(last)), ..RedrawState::default() }
    }

    #[test]
    fn redraw_rules() {
        let now = at("2026-11-25T10:03");
        assert_eq!(decide_redraw(now, &RedrawState { locked: true, dirty: true, ..drawn("2026-11-25T10:00") }), RedrawDecision::Hold);
        assert_eq!(decide_redraw(now, &RedrawState { paused: true, ..drawn("2026-11-25T09:00") }), RedrawDecision::Hold);
        assert_eq!(decide_redraw(now, &RedrawState { dirty: true, ..drawn("2026-11-25T10:02") }), RedrawDecision::Now);
        assert_eq!(decide_redraw(now, &RedrawState::default()), RedrawDecision::Now);
        assert_eq!(decide_redraw(now, &drawn("2026-11-25T10:00")), RedrawDecision::At(at("2026-11-25T10:05")));
        assert_eq!(decide_redraw(now, &RedrawState { battery_saver: true, ..drawn("2026-11-25T10:00") }), RedrawDecision::At(at("2026-11-25T10:15")));
        assert_eq!(decide_redraw(at("2026-11-25T10:05"), &drawn("2026-11-25T10:00")), RedrawDecision::Now);
    }

    #[test]
    fn midnight_forces_a_redraw() {
        assert_eq!(decide_redraw(at("2026-11-26T00:01"), &drawn("2026-11-25T23:58")), RedrawDecision::Now);
        assert_eq!(decide_redraw(at("2026-11-25T23:59"), &drawn("2026-11-25T23:58")), RedrawDecision::At(at("2026-11-26T00:00")));
    }
}
