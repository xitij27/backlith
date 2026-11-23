// SPDX-License-Identifier: GPL-3.0-or-later
//! Where Backlith's calendars come from: calendar links, fetched with a
//! cache, and the secret store that keeps those links out of every file.

pub mod fetch;
pub mod secrets;

pub use fetch::*;
pub use secrets::*;
