// SPDX-License-Identifier: GPL-3.0-or-later
//! Windows time zone names, as Outlook writes them into calendar files,
//! mapped to IANA zones. The table comes from CLDR: see `data/README.md`.

use std::collections::HashMap;
use std::sync::OnceLock;

use chrono_tz::Tz;

static TABLE: &str = include_str!("../data/windows-zones.csv");

fn table() -> &'static HashMap<&'static str, &'static str> {
    static MAP: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    MAP.get_or_init(|| TABLE.lines().skip(1).filter_map(|l| l.split_once(',')).collect())
}

/// `"W. Europe Standard Time"` → `Europe/Berlin`.
pub fn windows_zone_to_iana(name: &str) -> Option<Tz> {
    table().get(name.trim()).and_then(|iana| iana.parse().ok())
}

/// Every IANA zone the table maps to, in table order.
pub(crate) fn all_iana() -> Vec<&'static str> {
    let mut seen = std::collections::HashSet::new();
    TABLE.lines().skip(1).filter_map(|l| l.split_once(',')).map(|(_, iana)| iana).filter(|z| seen.insert(*z)).collect()
}
