// Copyright (c) 2023, Direct Decisions Rust client AUTHORS.
// All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//! # Direct Decisions API Client
//!
//! `ddclient-rs` is a Rust client library for interacting with the Direct Decisions API.
//! It provides a convenient way to access and manipulate voting data using the Direct Decisions API.
//!
//! The client supports various operations such as creating votings, voting, unvoting,
//! retrieving voting results, and more.
//!
//! The api specification can be found at https://api.directdecisions.com/v1.
//!
//! ## Features
//!
//! - Create and manage votings.
//! - Submit votes and retrieve ballots.
//! - Modify voting choices.
//! - Fetch voting results and analyze outcomes.
//! - Handle rate limits and errors gracefully.
//!
//! ## Usage
//!
//! To use `ddclient-rs`, add it as a dependency in your `Cargo.toml` file:
//!
//! ```toml
//! [dependencies]
//! ddclient-rs = "0.1.0"
//! ```
//!
//! Then, import `ddclient-rs` in your Rust file and use the `Client` struct to interact with the API.
//!
//! ```no_run
//! use ddclient_rs::Client;
//!
//! #[tokio::main]
//! async fn main() {
//!     let client = Client::builder("your-api-key".to_string()).build();
//!
//!     // Example: Creating a new voting
//!     let voting = client.create_voting(vec!["Einstein".to_string(), "Newton".to_string()]).await.unwrap();
//!     println!("Created voting: {:?}", voting);
//!
//! }
//! ```
//!
//! ## Error Handling
//!
//! The client uses custom error types defined in the `ddclient_rs::errors`, the APIError enum.
//!
//! ## Examples
//!
//! See the `examples/` directory for more example usage of the `ddclient-rs`.
//!
//! ## Contributions
//!
//! Contributions are welcome! Please refer to the repository's `CONTRIBUTING.md` file for contribution guidelines.
//!
mod client;
mod errors;
mod rate;

pub use client::*;
pub use errors::*;
pub use rate::Rate;
use reqwest::{Response, StatusCode};

use serde::{Deserialize, Serialize};

const CONTENT_TYPE: &str = "application/json; charset=utf-8";
const USER_AGENT: &str = "ddclient-rs/0.1.0";
const DEFAULT_BASE_URL: &str = "https://api.directdecisions.com";

/// Represents the results of a voting process.
///
/// This struct contains the overall results of a voting, including details on whether the
/// voting resulted in a tie and the individual results for each choice.
/// It can also contain additional information about how choices compare to each other in duels
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct VotingResults {
    pub tie: bool,
    pub results: Vec<VotingResult>,
    pub duels: Option<Vec<Duels>>,
}

/// Represents the duel information for 2 choices, as part of the voting results.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Duels {
    pub left: ChoiceStrength,
    pub right: ChoiceStrength,
}

/// Represents the strength of a choice compared to another choice in a duel.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ChoiceStrength {
    pub index: isize,
    pub choice: String,
    pub strength: isize,
}

/// Represents the single result for a specific choice.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct VotingResult {
    pub choice: String,
    pub index: i32,
    pub wins: i32,
    pub percentage: f32,
}

/// Represents a voting.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Voting {
    pub id: String,
    pub choices: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiErrorResponse {
    code: i32,
    message: String,
    errors: Vec<String>,
}

async fn handle_api_response<T: serde::de::DeserializeOwned>(
    response: Response,
) -> Result<T, ApiError> {
    match response.status() {
        StatusCode::OK => response
            .json()
            .await
            .map_err(|err| ApiError::Client(ClientError::HttpRequestError(err))),
        StatusCode::NOT_FOUND => Err(ApiError::NotFound),
        StatusCode::UNAUTHORIZED => Err(ApiError::Unauthorized),
        StatusCode::FORBIDDEN => Err(ApiError::Forbidden),
        StatusCode::TOO_MANY_REQUESTS => Err(ApiError::TooManyRequests),
        StatusCode::METHOD_NOT_ALLOWED => Err(ApiError::MethodNotAllowed),
        StatusCode::BAD_REQUEST => match response.json::<ApiErrorResponse>().await {
            Ok(error_resp) => {
                let bad_request_errors = error_resp
                    .errors
                    .into_iter()
                    .filter_map(|err| {
                        serde_json::from_str::<BadRequestError>(&format!("\"{}\"", err)).ok()
                    })
                    .collect();
                Err(ApiError::BadRequest(bad_request_errors))
            }
            Err(_) => Err(ApiError::BadRequest(vec![])),
        },
        StatusCode::SERVICE_UNAVAILABLE => Err(ApiError::Client(ClientError::ServiceUnavailable)),
        StatusCode::BAD_GATEWAY => Err(ApiError::Client(ClientError::BadGateway)),
        StatusCode::INTERNAL_SERVER_ERROR => {
            let error_message = response.text().await.unwrap_or_default();
            Err(ApiError::InternalServerError(error_message))
        }
        _ => {
            let error_message = response.text().await.unwrap_or_default();
            Err(ApiError::Other(error_message))
        }
    }
}
