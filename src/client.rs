// Copyright (c) 2023, Direct Decisions Rust client AUTHORS.
// All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use crate::{
    handle_api_response, ApiError, ClientError, Rate, Voting, VotingResults, CONTENT_TYPE,
    DEFAULT_BASE_URL, USER_AGENT,
};

use reqwest::{Method, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Serialize, Deserialize)]
struct VotingRequest {
    choices: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SetChoiceRequest {
    choice: String,
    index: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct SetChoiceResponse {
    choices: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct VoteResponse {
    revoted: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Ballot {
    ballot: HashMap<String, i32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OkResponse {
    code: i32,
    message: String,
}

/// A client for accessing the Direct Decisions API.
///
/// This struct provides methods to interact with various endpoints of the
/// Direct Decisions API, including creating votings, voting, and fetching results.
/// The api specification can be found at https://api.directdecisions.com/v1.
/// All possible Error responses are described in the ApiError enum and the above documentation.
///
/// # Examples
///
/// ```no_run
/// use ddclient_rs::Client;
///
/// #[tokio::main]
/// async fn main() {
///     let client = Client::new("my-api-key".to_string());
///     // Use client to interact with the API...
/// }
/// ```

pub struct Client {
    token: String,
    client: reqwest::Client,
    api_url: String,
    rate: Arc<Mutex<Option<Rate>>>,
}

impl Client {
    /// Constructs a new `Client` with the given API token, and the default API URL.
    /// The default API URL is `https://api.directdecisions.com`.
    /// If you need to use a custom API URL, use `Client::builder` instead.
    /// The default Reqwest client is created. If you need to use a custom Reqwest client,
    /// use `Client::builder` instead.
    /// Client parses and stores received rate limit information which is updated after each request.
    /// To access the rate limit information, use `Client::get_rate`.
    ///
    /// # Arguments
    ///
    /// * `token` - The API token used for authenticating with the Direct Decisions API.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ddclient_rs::Client;
    ///
    /// let client = Client::new("my-api-key".to_string());
    /// ```
    pub fn new(token: String) -> Self {
        Self::builder(token).build()
    }

    /// Creates a new `ClientBuilder` for constructing a `Client`.
    ///
    /// This method initializes a builder with the provided API token.
    /// Additional configurations, such as a custom API URL or Reqwest client,
    /// can be set using the builder's methods before building the `Client`.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use ddclient_rs::Client;
    ///
    /// let client = Client::builder("my-api-key".to_string())
    ///     .build();
    /// ```
    ///
    /// Advanced usage with custom configurations:
    ///
    /// ```
    /// use ddclient_rs::Client;
    ///
    /// let client = Client::builder("my-api-key".to_string())
    ///     .api_url("https://custom-api.directdecisions.com".to_string())
    ///     .build();
    /// ```
    pub fn builder(token: String) -> ClientBuilder {
        ClientBuilder::new(token)
    }

    /// Retrieves the current rate limit information.
    ///
    /// This method returns the most recent rate limit information as received
    /// from the Direct Decisions API, if available.
    /// If no rate limit information is available, `None` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use ddclient_rs::Client;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::builder("my-api-key".to_string())
    ///         .build();
    ///
    ///     if let Some(rate) = client.get_rate() {
    ///         println!("Current rate limit: {:?}", rate);
    ///     } else {
    ///         println!("No rate limit information available.");
    ///     }
    /// }
    /// ```
    pub fn get_rate(&self) -> Option<Rate> {
        let rate = self.rate.lock().unwrap();
        rate.clone()
    }

    async fn request<T: serde::Serialize>(
        &self,
        method: Method,
        path: &str,
        body: Option<T>,
    ) -> Result<Response, ClientError> {
        let url = format!("{}{}", self.api_url, path);

        let mut request = self
            .client
            .request(method, url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", CONTENT_TYPE)
            .header("User-Agent", USER_AGENT);

        if let Some(b) = body {
            request = request.header("Content-Type", CONTENT_TYPE);
            request = request.json(&b);
        }

        let response = request
            .send()
            .await
            .map_err(|err| ClientError::HttpRequestError(err.without_url()));

        if let Ok(response) = &response {
            let rate_update = Rate::from_headers(response.headers());
            let mut rate = self.rate.lock().unwrap();
            *rate = rate_update;
        }

        response
    }

    /// Creates a new voting.
    ///
    /// Sends a POST request to the Direct Decisions API to create a new voting
    /// with the specified choices.
    ///
    /// Returns a `Result` which is `Ok` containing the created `Voting` if successful,
    /// or an `Err` with an `ApiError` if the request fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ddclient_rs::Client;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::builder("my-api-key".to_string()).build();
    ///     let result = client.create_voting(vec!["Option 1".into(), "Option 2".into()]).await;
    ///     // Handle result...
    /// }
    /// ```
    pub async fn create_voting(&self, choices: Vec<String>) -> Result<Voting, ApiError> {
        let response = self
            .request(Method::POST, "v1/votings", Some(VotingRequest { choices }))
            .await?;

        handle_api_response(response).await
    }

    /// Retrieves a voting by its ID.
    ///
    /// Returns a `Result` which is `Ok` containing the `Voting` if found,
    /// or an `Err` with an `ApiError` if the voting is not found or the request fails.
    pub async fn get_voting(&self, id: &str) -> Result<Voting, ApiError> {
        let mut uri = "v1/votings/".to_string();
        url_escape::encode_path_to_string(id, &mut uri);

        let response = self.request::<Voting>(Method::GET, &uri, None).await?;

        handle_api_response(response).await
    }

    /// Deletes a voting by its ID.
    ///
    /// Returns a `Result` which is `Ok` if the voting was deleted successfully,
    /// or an `Err` with an `ApiError` if the voting is not found or the request fails.
    pub async fn delete_voting(&self, id: &str) -> Result<(), ApiError> {
        let mut uri = "v1/votings/".to_string();
        url_escape::encode_path_to_string(id, &mut uri);

        let response = self
            .request::<OkResponse>(Method::DELETE, &uri, None)
            .await?;

        let _ = handle_api_response::<OkResponse>(response).await?;

        Ok(())
    }

    /// Sets or updates a choice in a voting.
    //////
    /// This endpoint combines all possible modifications of the choices list elements.
    /// To add a new choice, provide its value as a string and an index where it should be placed in the list. For example, index 0 will append a new choice, while index equal to the number of choices will prepend it. For any other index number between, the choice will be inserted at that position.
    /// To remove a choice, provide the exact choice value as the string and set index to -1 value.
    /// To move an existing choice to a new position, provide the exact choice value as the string and its new position as the index.
    ///
    /// Returns a `Result` with the updated list of choices if successful,
    /// or an `Err` with an `ApiError` if the request fails.
    /// # Examples
    ///
    /// ```no_run
    /// use ddclient_rs::Client;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::builder("my-api-key".to_string()).build();
    ///     let result = client.set_choice("voting_id", "New Choice", 0).await;
    ///     // Handle result...
    /// }
    /// ```
    pub async fn set_choice(
        &self,
        voting_id: &str,
        choice: &str,
        index: i32,
    ) -> Result<Vec<String>, ApiError> {
        let mut uri = "v1/votings/".to_string();
        url_escape::encode_path_to_string(voting_id, &mut uri);
        uri.push_str("/choices");

        let response = self
            .request(
                Method::POST,
                &uri,
                Some(SetChoiceRequest {
                    choice: choice.to_string(),
                    index,
                }),
            )
            .await?;

        let resp = handle_api_response::<SetChoiceResponse>(response).await?;

        Ok(resp.choices)
    }

    /// Submits a vote on a specific voting.
    ///
    /// Votes are submitted as a ballot, which is a map of choices to their ranks.
    /// The ranks are integers starting from 1, where 1 is the highest rank.
    /// Not all choices need to be included in the ballot.
    ///
    /// Returns a `Result` which is `Ok` indicating whether the vote was a revote,
    /// or an `Err` with an `ApiError` if the voting is not found or the request fails.
    ///

    /// # Examples
    ///
    /// ```
    /// use ddclient_rs::Client;
    /// use std::collections::HashMap;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::builder("my-api-key".to_string()).build();
    ///     let ballot = HashMap::from([
    ///         ("Choice 1".to_string(), 1),
    ///         ("Choice 2".to_string(), 2),
    ///     ]);
    ///     let result = client.vote("voting_id", "voter_id", ballot).await;
    ///     // Handle result...
    /// }
    /// ```
    pub async fn vote(
        &self,
        voting_id: &str,
        voter_id: &str,
        ballot: HashMap<String, i32>,
    ) -> Result<bool, ApiError> {
        let mut uri = "v1/votings/".to_string();
        url_escape::encode_path_to_string(voting_id, &mut uri);
        uri.push_str("/ballots/");
        url_escape::encode_path_to_string(voter_id, &mut uri);

        let response = self
            .request(Method::POST, &uri, Some(Ballot { ballot }))
            .await?;

        let response = handle_api_response::<VoteResponse>(response).await?;

        Ok(response.revoted)
    }

    /// Removes a voter's ballot from a specific voting.
    pub async fn unvote(&self, voting_id: &str, voter_id: &str) -> Result<(), ApiError> {
        let mut uri = "v1/votings/".to_string();
        url_escape::encode_path_to_string(voting_id, &mut uri);
        uri.push_str("/ballots/");
        url_escape::encode_path_to_string(voter_id, &mut uri);

        let response = self
            .request::<OkResponse>(Method::DELETE, &uri, None)
            .await?;

        let _ = handle_api_response::<OkResponse>(response).await?;

        Ok(())
    }

    /// Retrieves a ballot for a specific voting and voter.
    /// The ballot is returned as a map of choices to their ranks.
    /// The ranks are integers starting from 1, where 1 is the highest rank.
    pub async fn get_ballot(
        &self,
        voting_id: &str,
        voter_id: &str,
    ) -> Result<HashMap<String, i32>, ApiError> {
        let mut uri = "v1/votings/".to_string();
        url_escape::encode_path_to_string(voting_id, &mut uri);
        uri.push_str("/ballots/");
        url_escape::encode_path_to_string(voter_id, &mut uri);

        let response = self.request::<Ballot>(Method::GET, &uri, None).await?;

        let response = handle_api_response::<Ballot>(response).await?;

        Ok(response.ballot)
    }

    /// Retrieves the results of a specific voting.
    /// The results are returned as a list of choices with their wins, percentage, and index.
    pub async fn get_voting_results(&self, voting_id: &str) -> Result<VotingResults, ApiError> {
        let mut uri = "v1/votings/".to_string();
        url_escape::encode_path_to_string(voting_id, &mut uri);
        uri.push_str("/results");

        let response = self
            .request::<VotingResults>(Method::GET, &uri, None)
            .await?;

        handle_api_response(response).await
    }
}

/// A builder for creating an instance of `Client`.
///
/// This builder allows for configuring optional parameters for `Client`,
/// such as a custom API URL or a custom Reqwest client.
///
/// # Examples
///
/// ```
/// use ddclient_rs::{Client, ClientBuilder};
///
/// let client = Client::builder("my-api-key".to_string())
///     .api_url("https://custom-api.directdecisions.com".to_string())
///     .build();
/// ```
pub struct ClientBuilder {
    token: String,
    api_url: Option<String>,
    reqwest_client: Option<reqwest::Client>,
}

impl ClientBuilder {
    fn new(token: String) -> Self {
        ClientBuilder {
            token,
            api_url: None,
            reqwest_client: None,
        }
    }

    /// Sets a custom API URL for the `Client`.
    ///
    /// If not set, a default URL is used.
    ///
    /// # Arguments
    ///
    /// * `api_url` - A string representing the custom API URL.
    pub fn api_url(mut self, api_url: String) -> Self {
        self.api_url = Some(api_url);
        self
    }

    /// Sets a custom Reqwest client for the `Client`.
    ///
    /// If not set, a default Reqwest client is used.
    ///
    /// # Arguments
    ///
    /// * `client` - An instance of `reqwest::Client` to be used with the `Client`.
    pub fn reqwest_client(mut self, client: reqwest::Client) -> Self {
        self.reqwest_client = Some(client);
        self
    }

    /// Builds and returns a new `Client` instance.
    ///
    /// This method consumes the builder, applies URL validation and formatting,
    /// and uses the provided configurations to create a `Client`.
    /// If certain configurations are not provided, default values are used.
    ///
    /// # Panics
    ///
    /// Panics if the provided API URL is invalid.
    ///
    /// # Returns
    ///
    /// Returns a `Client` instance with the configured options.
    ///
    /// # Examples
    ///
    /// ```
    /// use ddclient_rs::Client;
    ///
    /// let client = Client::builder("my-api-key".to_string())
    ///     .api_url("https://custom-api.directdecisions.com".to_string())
    ///     .build();
    /// ```
    pub fn build(self) -> Client {
        let mut api_url = match self.api_url {
            Some(url) => {
                let _ = reqwest::Url::parse(&url).expect("Invalid API URL");
                url
            }
            None => DEFAULT_BASE_URL.to_string(),
        };

        if !api_url.ends_with('/') {
            api_url.push('/');
        }

        let client = self.reqwest_client.unwrap_or_default();

        Client {
            token: self.token,
            client,
            api_url,
            rate: Arc::new(Mutex::new(None)),
        }
    }
}
