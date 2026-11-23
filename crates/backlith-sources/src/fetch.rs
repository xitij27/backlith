// SPDX-License-Identifier: GPL-3.0-or-later
//! Fetching calendar links, with a cache. A failed fetch never empties a
//! calendar: the last good copy stays. No error text ever names the link,
//! because the link is a secret.

use std::path::PathBuf;

use backlith_core::{read_ics, DateRange};
use chrono::{NaiveDate, NaiveDateTime};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LinkError {
    #[error("That doesn't look like a calendar link. It should start with https:// or webcal://.")]
    NotALink,
}

/// Trims the link and turns `webcal://` into `https://`. Only http(s) links are accepted.
pub fn normalize_link(input: &str) -> Result<String, LinkError> {
    let t = input.trim();
    let lower = t.to_ascii_lowercase();
    let link = if let Some(n) = ["webcals://", "webcal://"].iter().find(|p| lower.starts_with(**p)).map(|p| p.len()) {
        format!("https://{}", &t[n..])
    } else if lower.starts_with("https://") || lower.starts_with("http://") {
        t.to_string()
    } else {
        return Err(LinkError::NotALink);
    };
    match reqwest::Url::parse(&link) {
        Ok(url) if url.host_str().is_some_and(|h| !h.is_empty()) => Ok(link),
        _ => Err(LinkError::NotALink),
    }
}

/// Calendars larger than this are refused.
pub const MAX_CALENDAR_BYTES: usize = 20 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheMeta {
    pub fetched_at: NaiveDateTime,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchOutcome {
    Updated,
    NotModified,
    /// `reason` never contains the link.
    Failed { reason: String },
}

enum Download {
    NotModified,
    Body { text: String, etag: Option<String>, last_modified: Option<String> },
}

/// A failure, in words that never repeat the link: reqwest's own messages
/// can contain the URL, so only the kind of error is used.
fn describe(e: &reqwest::Error) -> String {
    let words = if e.is_timeout() {
        "the calendar server did not answer in time"
    } else if e.is_connect() {
        "could not reach the calendar server"
    } else if e.is_body() || e.is_decode() {
        "the calendar download broke off"
    } else {
        "the request to the calendar server failed"
    };
    words.to_string()
}

fn header(resp: &reqwest::Response, name: reqwest::header::HeaderName) -> Option<String> {
    resp.headers().get(name).and_then(|v| v.to_str().ok()).map(str::to_string)
}

/// Source ids become file names, so keep only safe characters.
fn file_stem(source_id: &str) -> String {
    source_id.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect()
}

pub struct Fetcher {
    client: reqwest::Client,
    cache_dir: PathBuf,
}

impl Fetcher {
    pub fn new(cache_dir: PathBuf) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(20))
            .user_agent(concat!("Backlith/", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("an HTTP client with these settings can always be built");
        Self { client, cache_dir }
    }

    fn body_path(&self, source_id: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.ics", file_stem(source_id)))
    }

    fn meta_path(&self, source_id: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.json", file_stem(source_id)))
    }

    async fn download(&self, url: &str, since: Option<&CacheMeta>) -> Result<Download, String> {
        use reqwest::header::{ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED};
        let mut req = self.client.get(url);
        if let Some(meta) = since {
            if let Some(etag) = &meta.etag {
                req = req.header(IF_NONE_MATCH, etag);
            }
            if let Some(lm) = &meta.last_modified {
                req = req.header(IF_MODIFIED_SINCE, lm);
            }
        }
        let resp = req.send().await.map_err(|e| describe(&e))?;
        let status = resp.status();
        if status == reqwest::StatusCode::NOT_MODIFIED && since.is_some() {
            return Ok(Download::NotModified);
        }
        if !status.is_success() {
            return Err(format!("the calendar server answered {}", status.as_u16()));
        }
        if resp.content_length().is_some_and(|n| n > MAX_CALENDAR_BYTES as u64) {
            return Err("the calendar is larger than 20 MB".into());
        }
        let etag = header(&resp, ETAG);
        let last_modified = header(&resp, LAST_MODIFIED);
        let bytes = resp.bytes().await.map_err(|e| describe(&e))?;
        if bytes.len() > MAX_CALENDAR_BYTES {
            return Err("the calendar is larger than 20 MB".into());
        }
        Ok(Download::Body { text: String::from_utf8_lossy(&bytes).into_owned(), etag, last_modified })
    }

    fn store(&self, source_id: &str, text: &str, meta: &CacheMeta) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.cache_dir)?;
        let body = self.body_path(source_id);
        let tmp = body.with_extension("ics.tmp");
        std::fs::write(&tmp, text)?;
        std::fs::rename(&tmp, &body)?;
        backlith_core::save_json(&self.meta_path(source_id), meta).map_err(std::io::Error::other)
    }

    pub async fn fetch(&self, source_id: &str, url: &str, now: NaiveDateTime) -> FetchOutcome {
        let meta = self.cache_meta(source_id);
        let since = if self.body_path(source_id).exists() { meta.as_ref() } else { None };
        match self.download(url, since).await {
            Err(reason) => FetchOutcome::Failed { reason },
            Ok(Download::NotModified) => {
                if let Some(mut m) = meta {
                    m.fetched_at = now;
                    let _ = backlith_core::save_json(&self.meta_path(source_id), &m);
                }
                FetchOutcome::NotModified
            }
            Ok(Download::Body { text, etag, last_modified }) => {
                let today = now.date();
                let range = DateRange { from: today, to: today + chrono::Duration::days(1) };
                if read_ics(&text, source_id, chrono_tz::UTC, &range).is_err() {
                    return FetchOutcome::Failed { reason: "the link no longer returns a calendar".into() };
                }
                let meta = CacheMeta { fetched_at: now, etag, last_modified };
                match self.store(source_id, &text, &meta) {
                    Ok(()) => FetchOutcome::Updated,
                    Err(_) => FetchOutcome::Failed { reason: "could not save the calendar on this computer".into() },
                }
            }
        }
    }

    pub fn cached_text(&self, source_id: &str) -> Option<String> {
        std::fs::read_to_string(self.body_path(source_id)).ok()
    }

    pub fn cache_meta(&self, source_id: &str) -> Option<CacheMeta> {
        backlith_core::load_json(&self.meta_path(source_id)).ok().flatten()
    }

    /// Removes a source's cached copy.
    pub fn forget(&self, source_id: &str) {
        let _ = std::fs::remove_file(self.body_path(source_id));
        let _ = std::fs::remove_file(self.meta_path(source_id));
    }

    /// How many events the link has in the next 8 weeks, to show when it is added.
    pub async fn check(&self, url: &str, viewer_tz: Tz, today: NaiveDate) -> Result<usize, String> {
        match self.download(url, None).await? {
            Download::NotModified => Err("the calendar server answered 304".into()),
            Download::Body { text, .. } => {
                let range = DateRange { from: today, to: today + chrono::Duration::weeks(8) };
                read_ics(&text, "check", viewer_tz, &range)
                    .map(|events| events.len())
                    .map_err(|_| "the link does not return a calendar".to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn webcal_becomes_https() {
        assert_eq!(normalize_link("  webcal://x.example/a.ics ").unwrap(), "https://x.example/a.ics");
        assert_eq!(normalize_link("WEBCAL://x.example/a.ics").unwrap(), "https://x.example/a.ics");
        assert_eq!(normalize_link("https://calendar.google.com/calendar/ical/x/basic.ics").unwrap(), "https://calendar.google.com/calendar/ical/x/basic.ics");
        assert_eq!(normalize_link("ftp://x.example/a.ics"), Err(LinkError::NotALink));
        assert_eq!(normalize_link("hello"), Err(LinkError::NotALink));
        assert_eq!(normalize_link("https://"), Err(LinkError::NotALink));
    }
}
