use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT_LANGUAGE, HeaderMap, HeaderValue, USER_AGENT};

pub fn fetch_html(url: &str) -> Result<String> {
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static(
            "atcoder-tui-viewer/0.1 (+https://github.com/hxrxchang/atcoder-tui-viewer)",
        ),
    );
    headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("ja,en;q=0.8"));

    let client = Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(15))
        .build()
        .context("failed to build HTTP client")?;

    let response = client
        .get(url)
        .send()
        .context("request failed")?
        .error_for_status()
        .context("server returned an error status")?;

    response.text().context("failed to read response body")
}
