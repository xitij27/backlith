// SPDX-License-Identifier: GPL-3.0-or-later
//! Backlith's logic that does not touch the system: the week, calendars,
//! schedules, layouts and icon zones. Everything here compiles for Windows
//! and Linux alike and is tested on Linux.

pub mod icons;
pub mod ics;
pub mod layout;
pub mod redraw;
pub mod schedule;
pub mod settings;
pub mod week;
pub mod zones_windows;

pub use icons::*;
pub use ics::*;
pub use layout::*;
pub use redraw::*;
pub use schedule::*;
pub use settings::*;
pub use week::*;
pub use zones_windows::*;
