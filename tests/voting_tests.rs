// Copyright (c) 2023, Direct Decisions Rust client AUTHORS.
// All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use ddclient_rs::{ApiError, BadRequestError, Client, VotingResult};
use httpmock::prelude::*;
use httpmock::Mock;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

const CONTENT_TYPE: &str = "application/json; charset=utf-8";

fn request_mock(
    server: &MockServer,
    method: httpmock::Method,
    path: String,
    status: u16,
    req_body: Option<Value>,
    resp_body: Value,
) -> Mock {
    if let Some(body) = req_body {
        server.mock(|when, then| {
            when.method(method)
                .path(path)
                .header("Authorization", "Bearer test-token")
                .header("Accept", CONTENT_TYPE)
                .header("Content-Type", CONTENT_TYPE)
                .json_body(body);

            then.status(status)
                .header("Content-Type", CONTENT_TYPE)
                .json_body(resp_body);
        })
    } else {
        server.mock(|when, then| {
            when.method(method)
                .path(path)
                .header("Authorization", "Bearer test-token")
                .header("Accept", CONTENT_TYPE);

            then.status(status)
                .header("Content-Type", CONTENT_TYPE)
                .json_body(json!(&resp_body));
        })
    }
}

fn prepare_client_server() -> (MockServer, Client) {
    let server = MockServer::start();
    let client = Client::builder("test-token".to_string())
        .api_url(server.base_url())
        .build();
    (server, client)
}

#[tokio::test]
async fn create_voting_test() {
    let (server, client) = prepare_client_server();

    let mock = request_mock(
        &server,
        POST,
        "/v1/votings".to_string(),
        200,
        Some(json!({"choices":["Spinoza","Kant","Nietzsche"]})),
        json!({"id":"40f80454800b2bd7c172","choices":["Spinoza","Kant","Nietzsche"]}),
    );

    let got_voting = client
        .create_voting(vec![
            "Spinoza".to_string(),
            "Kant".to_string(),
            "Nietzsche".to_string(),
        ])
        .await
        .unwrap();

    assert_eq!(got_voting.id, "40f80454800b2bd7c172");
    assert_eq!(got_voting.choices, vec!["Spinoza", "Kant", "Nietzsche"]);
    mock.assert();
}

#[tokio::test]
async fn get_voting_test() {
    let (server, client) = prepare_client_server();

    let mock = request_mock(
        &server,
        GET,
        "/v1/votings/40f80454800b2bd7c172".to_string(),
        200,
        None,
        json!({"id":"40f80454800b2bd7c172","choices":["Spinoza","Kant","Nietzsche"]}),
    );

    let got_voting = client.get_voting("40f80454800b2bd7c172").await.unwrap();
    assert_eq!(got_voting.id, "40f80454800b2bd7c172");
    assert_eq!(got_voting.choices, vec!["Spinoza", "Kant", "Nietzsche"]);
    mock.assert();
}

#[tokio::test]
async fn delete_voting_test() {
    let (server, client) = prepare_client_server();

    let mock = request_mock(
        &server,
        DELETE,
        "/v1/votings/40f80454800b2bd7c172".to_string(),
        200,
        None,
        // ok response
        json!({"code":200,"message":"OK"}),
    );

    client.delete_voting("40f80454800b2bd7c172").await.unwrap();
    mock.assert();
}

#[tokio::test]
async fn set_choice_test() {
    let (server, client) = prepare_client_server();

    let mock = request_mock(
        &server,
        POST,
        "/v1/votings/40f80454800b2bd7c172/choices".to_string(),
        200,
        Some(json!({"choice":"Schopenhauer", "index": 0})),
        json!({"id":"40f80454800b2bd7c172","choices":["Schopenhauer","Spinoza","Kant","Nietzsche"]}),
    );

    // index 0 means append to the beginning of the list
    let got_choices = client
        .set_choice("40f80454800b2bd7c172", "Schopenhauer", 0)
        .await
        .unwrap();

    assert_eq!(got_choices[0], "Schopenhauer");
    assert_eq!(got_choices[1], "Spinoza");
    assert_eq!(got_choices[2], "Kant");
    assert_eq!(got_choices[3], "Nietzsche");

    mock.assert();
}

#[tokio::test]
async fn vote_test() {
    let (server, client) = prepare_client_server();

    #[derive(Debug, Serialize, Deserialize)]
    struct Ballot {
        ballot: HashMap<String, i32>,
    }

    let ballot = Ballot {
        ballot: HashMap::from([
            ("Schopenhauer".to_string(), 1),
            ("Spinoza".to_string(), 1),
            ("Kant".to_string(), 1),
            ("Nietzsche".to_string(), 1),
        ]),
    };

    let mock = request_mock(
        &server,
        POST,
        "/v1/votings/40f80454800b2bd7c172/ballots/einstein".to_string(),
        200,
        Some(json!(ballot)),
        json!({"revoted": false}),
    );

    let revoted = client
        .vote("40f80454800b2bd7c172", "einstein", ballot.ballot)
        .await
        .unwrap();
    assert_eq!(revoted, false);
    mock.assert();
}

#[tokio::test]
async fn unvote_test() {
    let (server, client) = prepare_client_server();

    let mock = request_mock(
        &server,
        DELETE,
        "/v1/votings/40f80454800b2bd7c172/ballots/einstein".to_string(),
        200,
        None,
        json!({"code":200,"message":"OK"}),
    );

    let _ = client
        .unvote("40f80454800b2bd7c172", "einstein")
        .await
        .unwrap();
    mock.assert();
}

#[tokio::test]
async fn get_ballot_test() {
    let (server, client) = prepare_client_server();

    let mock = request_mock(
        &server,
        GET,
        "/v1/votings/40f80454800b2bd7c172/ballots/einstein".to_string(),
        200,
        None,
        json!({"ballot":{"Schopenhauer":1,"Spinoza":1,"Kant":1,"Nietzsche":1}}),
    );

    let got_ballot = client
        .get_ballot("40f80454800b2bd7c172", "einstein")
        .await
        .unwrap();
    assert_eq!(got_ballot["Schopenhauer"], 1);
    assert_eq!(got_ballot["Spinoza"], 1);
    assert_eq!(got_ballot["Kant"], 1);
    assert_eq!(got_ballot["Nietzsche"], 1);
    mock.assert();
}

#[tokio::test]
async fn get_voting_results_test() {
    let (server, client) = prepare_client_server();

    let voting_results = ddclient_rs::VotingResults {
        results: vec![
            VotingResult {
                choice: "Schopenhauer".to_string(),
                index: 0,
                wins: 1,
                percentage: 50.0,
                strength: 1,
                advantage: 0,
            },
            VotingResult {
                choice: "Spinoza".to_string(),
                index: 1,
                wins: 1,
                percentage: 50.0,
                strength: 1,
                advantage: 0,
            },
            VotingResult {
                choice: "Kant".to_string(),
                index: 2,
                wins: 1,
                percentage: 50.0,
                strength: 1,
                advantage: 0,
            },
            VotingResult {
                choice: "Nietzsche".to_string(),
                index: 3,
                wins: 1,
                percentage: 50.0,
                strength: 1,
                advantage: 0,
            },
        ],
        tie: true,
        duels: None,
    };

    let mock = request_mock(
        &server,
        GET,
        "/v1/votings/40f80454800b2bd7c172/results".to_string(),
        200,
        None,
        json!(voting_results),
    );

    let got_results = client
        .get_voting_results("40f80454800b2bd7c172")
        .await
        .unwrap();

    assert_eq!(got_results, voting_results);
    mock.assert();
}

#[tokio::test]
async fn get_voting_results_duels_test() {
    let (server, client) = prepare_client_server();

    let voting_results = ddclient_rs::VotingResults {
        results: vec![
            VotingResult {
                choice: "Schopenhauer".to_string(),
                index: 0,
                wins: 1,
                percentage: 50.0,
                strength: 1,
                advantage: 0,
            },
            VotingResult {
                choice: "Spinoza".to_string(),
                index: 1,
                wins: 1,
                percentage: 50.0,
                strength: 1,
                advantage: 0,
            },
            VotingResult {
                choice: "Kant".to_string(),
                index: 2,
                wins: 1,
                percentage: 50.0,
                strength: 1,
                advantage: 0,
            },
            VotingResult {
                choice: "Nietzsche".to_string(),
                index: 3,
                wins: 1,
                percentage: 50.0,
                strength: 1,
                advantage: 0,
            },
        ],
        tie: true,
        duels: Some(vec![
            ddclient_rs::Duels {
                left: ddclient_rs::ChoiceStrength {
                    index: 0,
                    choice: "Schopenhauer".to_string(),
                    strength: 1,
                },
                right: ddclient_rs::ChoiceStrength {
                    index: 1,
                    choice: "Spinoza".to_string(),
                    strength: 1,
                },
            },
            ddclient_rs::Duels {
                left: ddclient_rs::ChoiceStrength {
                    index: 2,
                    choice: "Kant".to_string(),
                    strength: 1,
                },
                right: ddclient_rs::ChoiceStrength {
                    index: 3,
                    choice: "Nietzsche".to_string(),
                    strength: 1,
                },
            },
        ]),
    };

    let mock = request_mock(
        &server,
        GET,
        "/v1/votings/40f80454800b2bd7c172/results".to_string(),
        200,
        None,
        json!(voting_results),
    );

    let got_results = client
        .get_voting_results("40f80454800b2bd7c172")
        .await
        .unwrap();

    assert_eq!(got_results, voting_results);
    mock.assert();
}

#[tokio::test]
async fn error_test() {
    let (server, client) = prepare_client_server();

    let mut mock = request_mock(
        &server,
        GET,
        "/v1/votings/40f80454800b2bd7c172".to_string(),
        404,
        None,
        json!({"code":404,"message":"Not Found"}),
    );

    let got_err = client.get_voting("40f80454800b2bd7c172").await.unwrap_err();
    assert!(matches!(got_err, ApiError::NotFound));
    mock.assert();
    mock.delete();

    // test bad request as well with InvalidData in the errors array
    let mock = request_mock(
        &server,
        GET,
        "/v1/votings/40f80454800b2bd7c172".to_string(),
        400,
        None,
        json!({"code":400,"message":"Bad Request","errors":["InvalidData"]}),
    );

    let got_err = client.get_voting("40f80454800b2bd7c172").await.unwrap_err();
    match got_err {
        ApiError::BadRequest(errors) => {
            assert_eq!(errors.len(), 1);
            assert!(matches!(errors[0], BadRequestError::InvalidData));
        }
        err => panic!("Expected BadRequest error {:?}", err),
    }

    mock.assert()
}

#[tokio::test]
async fn rate_test() {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    let (server, client) = prepare_client_server();

    let mut mock = request_mock(
        &server,
        GET,
        "/v1/votings/40f80454800b2bd7c172".to_string(),
        200,
        None,
        json!({"id":"40f80454800b2bd7c172","choices":["Spinoza","Kant","Nietzsche"]}),
    );

    let _ = client.get_voting("40f80454800b2bd7c172").await.unwrap();
    mock.assert();
    mock.delete();

    assert_eq!(true, client.get_rate().is_none());

    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/v1/votings/40f80454800b2bd7c172".to_string())
            .header("Authorization", "Bearer test-token")
            .header("Accept", CONTENT_TYPE);

        then.status(200)
            .header("Content-Type", CONTENT_TYPE)
            .header("X-RateLimit-Limit", "100")
            .header("X-RateLimit-Remaining", "50")
            .header("X-RateLimit-Reset", "1000")
            .header("Retry-After", "1000")
            .json_body(
                json!({"id":"40f80454800b2bd7c172","choices":["Spinoza","Kant","Nietzsche"]}),
            );
    });

    let voting = client.get_voting("40f80454800b2bd7c172").await.unwrap();
    assert_eq!(voting.id, "40f80454800b2bd7c172");
    assert_eq!(voting.choices, vec!["Spinoza", "Kant", "Nietzsche"]);

    let rate = client.get_rate().unwrap();
    assert_eq!(rate.limit, 100);
    assert_eq!(rate.remaining, 50);
    let now = SystemTime::now().duration_since(UNIX_EPOCH).ok().unwrap();
    let reset = now + Duration::from_secs(1000);
    assert_eq!(rate.reset, reset.as_secs());
    let retry = now + Duration::from_secs(1000);
    assert_eq!(rate.retry, retry.as_secs());
    mock.assert();
}
