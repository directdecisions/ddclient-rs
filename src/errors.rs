// Copyright (c) 2023, Direct Decisions Rust client AUTHORS.
// All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use serde::Deserialize;
use thiserror::Error;

/// Represents an error returned by the API.
///
/// This enum represents an error returned by the API. It contains various error types that can be
/// defined at https://api.directdecisions.com/v1.
///
/// Client errors represent errors that occur on the client side.
///
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Bad Request: {0:?}")]
    BadRequest(Vec<BadRequestError>),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Not Found")]
    NotFound,

    #[error("Forbidden")]
    Forbidden,

    #[error("Internal Server Error: {0}")]
    InternalServerError(String),

    #[error("Method Not Allowed")]
    MethodNotAllowed,

    #[error("Too many requests")]
    TooManyRequests,

    #[error("Other Error: {0}")]
    Other(String),

    #[error("Client Error: {0}")]
    Client(#[from] ClientError),
}

/// Represents a client error.
///
/// This enum represents a client error, such as a bad gateway or service unavailable error.
///
/// It also includes an HTTP request error variant that wraps the reqwest::Error type.
#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Bad Gateway")]
    BadGateway,
    #[error("HTTP Request Error: {0}")]
    HttpRequestError(#[from] reqwest::Error),

    #[error("Service Unavailable")]
    ServiceUnavailable,
}

/// Represents a bad request error.
#[derive(Error, Debug, Deserialize, PartialEq)]
pub enum BadRequestError {
    #[error("Invalid data")]
    InvalidData,
    #[error("Missing choices")]
    MissingChoices,
    #[error("Choice too long")]
    ChoiceTooLong,
    #[error("Too many choices")]
    TooManyChoices,
    #[error("Choice required")]
    ChoiceRequired,
    #[error("Ballot required")]
    BallotRequired,
    #[error("Voter ID too long")]
    VoterIDTooLong,
    #[error("Invalid voter ID")]
    InvalidVoterID,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handle_api_response;
    use http::response::Builder;
    use reqwest::{Response, StatusCode};

    impl PartialEq for ApiError {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (ApiError::BadRequest(errors_self), ApiError::BadRequest(errors_other)) => {
                    for err in errors_self {
                        if !errors_other.contains(err) {
                            return false;
                        }
                    }

                    true
                }
                (ApiError::Unauthorized, ApiError::Unauthorized) => true,
                (ApiError::NotFound, ApiError::NotFound) => true,
                (ApiError::Forbidden, ApiError::Forbidden) => true,
                (
                    ApiError::InternalServerError(msg_self),
                    ApiError::InternalServerError(msg_other),
                ) => msg_self == msg_other,
                (ApiError::MethodNotAllowed, ApiError::MethodNotAllowed) => true,
                (ApiError::TooManyRequests, ApiError::TooManyRequests) => true,
                (ApiError::Other(msg_self), ApiError::Other(msg_other)) => msg_self == msg_other,
                (ApiError::Client(err_self), ApiError::Client(err_other)) => {
                    match (err_self, err_other) {
                        (ClientError::BadGateway, ClientError::BadGateway) => true,
                        (ClientError::ServiceUnavailable, ClientError::ServiceUnavailable) => true,
                        (
                            ClientError::HttpRequestError(err_self),
                            ClientError::HttpRequestError(err_other),
                        ) => err_self.to_string() == err_other.to_string(),
                        _ => false,
                    }
                }
                _ => false,
            }
        }
    }

    fn create_mock_response(status: StatusCode, body: &str) -> Response {
        let response = Builder::new()
            .status(status)
            .body(body.to_string())
            .unwrap();
        Response::from(response)
    }

    #[tokio::test]
    async fn api_errors_test() {
        let test_cases = vec![
            (
                StatusCode::BAD_GATEWAY,
                "",
                ApiError::Client(ClientError::BadGateway),
            ),
            (
                StatusCode::SERVICE_UNAVAILABLE,
                "",
                ApiError::Client(ClientError::ServiceUnavailable),
            ),
            (StatusCode::BAD_REQUEST, "", ApiError::BadRequest(vec![])),
            (
                StatusCode::BAD_REQUEST,
                r#"{"code":400,"message":"Bad Request","errors":["Invalid data"]}"#,
                ApiError::BadRequest(vec![BadRequestError::InvalidData]),
            ),
            (
                StatusCode::BAD_REQUEST,
                r#"{"code":400,"message":"Bad Request","errors":["Missing choices"]}"#,
                ApiError::BadRequest(vec![BadRequestError::MissingChoices]),
            ),
            (
                StatusCode::BAD_REQUEST,
                r#"{"code":400,"message":"Bad Request","errors":["Choice too long"]}"#,
                ApiError::BadRequest(vec![BadRequestError::ChoiceTooLong]),
            ),
            (
                StatusCode::BAD_REQUEST,
                r#"{"code":400,"message":"Bad Request","errors":["Too many choices"]}"#,
                ApiError::BadRequest(vec![BadRequestError::TooManyChoices]),
            ),
            (
                StatusCode::BAD_REQUEST,
                r#"{"code":400,"message":"Bad Request","errors":["Choice required"]}"#,
                ApiError::BadRequest(vec![BadRequestError::ChoiceRequired]),
            ),
            (
                StatusCode::BAD_REQUEST,
                r#"{"code":400,"message":"Bad Request","errors":["Ballot required"]}"#,
                ApiError::BadRequest(vec![BadRequestError::BallotRequired]),
            ),
            (
                StatusCode::BAD_REQUEST,
                r#"{"code":400,"message":"Bad Request","errors":["Voter ID too long"]}"#,
                ApiError::BadRequest(vec![BadRequestError::VoterIDTooLong]),
            ),
            (
                StatusCode::BAD_REQUEST,
                r#"{"code":400,"message":"Bad Request","errors":["Invalid voter ID"]}"#,
                ApiError::BadRequest(vec![BadRequestError::InvalidVoterID]),
            ),
            (
                StatusCode::BAD_REQUEST,
                r#"{"code":400,"message":"Bad Request","errors":["Invalid data","Missing choices"]}"#,
                ApiError::BadRequest(vec![
                    BadRequestError::InvalidData,
                    BadRequestError::MissingChoices,
                ]),
            ),
            (StatusCode::TOO_MANY_REQUESTS, "", ApiError::TooManyRequests),
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error",
                ApiError::InternalServerError("Internal Server Error".to_string()),
            ),
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "",
                ApiError::InternalServerError("".to_string()),
            ),
            (StatusCode::NOT_FOUND, "", ApiError::NotFound),
            (StatusCode::UNAUTHORIZED, "", ApiError::Unauthorized),
            (StatusCode::FORBIDDEN, "", ApiError::Forbidden),
            (
                StatusCode::METHOD_NOT_ALLOWED,
                "",
                ApiError::MethodNotAllowed,
            ),
        ];

        for (status, body, expected_error) in test_cases {
            let mock_response = create_mock_response(status, body);
            let result = handle_api_response::<()>(mock_response).await;

            match result {
                Ok(_) => assert!(false, "Expected error but got Ok"),
                Err(err) => assert_eq!(err, expected_error),
            }
        }
    }
}
