// Copyright (c) 2023, Direct Decisions Rust client AUTHORS.
// All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

const HEADER_RATE_LIMIT: &str = "X-RateLimit-Limit";
const HEADER_RATE_REMAINING: &str = "X-RateLimit-Remaining";
const HEADER_RATE_RESET: &str = "X-RateLimit-Reset";
const HEADER_RATE_RETRY: &str = "Retry-After";

use reqwest::header::HeaderMap;
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Represents the rate limit information returned by the API.
///
/// This struct contains the rate limit information returned by the API, including the number of
/// requests allowed, the number of requests remaining, and the time at which the rate limit will
/// reset.
///
#[derive(Clone, Default, Debug)]
pub struct Rate {
    pub limit: u32,
    pub remaining: u32,
    pub reset: u64,
    pub retry: u64,
}

impl Rate {
    pub(crate) fn from_headers(headers: &HeaderMap) -> Option<Self> {
        let limit = fetch_header(headers, HEADER_RATE_LIMIT)?;
        let remaining = fetch_header(headers, HEADER_RATE_REMAINING)?;
        let reset_secs: u64 = fetch_header(headers, HEADER_RATE_RESET)?;
        let retry_secs: u64 = fetch_header(headers, HEADER_RATE_RETRY)?;

        let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?;
        let reset = now + Duration::from_secs(reset_secs);
        let retry = now + Duration::from_secs(retry_secs);

        Some(Self {
            limit,
            remaining,
            reset: reset.as_secs(),
            retry: retry.as_secs(),
        })
    }
}

fn fetch_header<T>(headers: &HeaderMap, header: &str) -> Option<T>
where
    T: FromStr,
{
    headers.get(header)?.to_str().ok()?.parse::<T>().ok()
}

// rate tests
#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::Method::GET;
    use httpmock::MockServer;

    #[tokio::test]
    async fn test_rate_from_headers() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/test");
            then.status(200)
                .header(HEADER_RATE_LIMIT, "100")
                .header(HEADER_RATE_REMAINING, "50")
                .header(HEADER_RATE_RESET, "1000")
                .header(HEADER_RATE_RETRY, "1000");
        });

        let client = reqwest::Client::new();
        let response = client.get(server.url("/test")).send().await.unwrap();
        let rate = Rate::from_headers(response.headers()).unwrap();

        assert_eq!(rate.limit, 100);
        assert_eq!(rate.remaining, 50);
        // assert that rate is time.now + reset from seconds rounded to 5 seconds

        let now = SystemTime::now().duration_since(UNIX_EPOCH).ok().unwrap();
        let reset = now + Duration::from_secs(1000);
        assert_eq!(rate.reset, reset.as_secs());
        let retry = now + Duration::from_secs(1000);
        assert_eq!(rate.retry, retry.as_secs());
        mock.assert();
    }

    #[tokio::test]
    async fn test_no_headers() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/test");
            then.status(200);
        });

        let client = reqwest::Client::new();
        let response = client.get(server.url("/test")).send().await.unwrap();
        let rate = Rate::from_headers(response.headers());
        assert!(rate.is_none());
        mock.assert();
    }
}
