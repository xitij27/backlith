// SPDX-License-Identifier: GPL-3.0-or-later
//! Backlith's logic that does not touch the system: the week, calendars,
//! schedules, layouts and icon zones. Everything here compiles for Windows
//! and Linux alike and is tested on Linux.

pub mod ics;
pub mod week;
pub mod zones_windows;

pub use ics::*;
pub use week::*;
pub use zones_windows::*;
