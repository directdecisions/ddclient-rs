# Direct Decisions API v1 Rust client

A client library for accessing the [Direct Decisions](https://directdecisions.com) v1 API.

You can view Direct Decisions API v1 docs here: [https://api.directdecisions.com/v1](https://api.directdecisions.com/v1)

This is an asynchronous Rust client using `reqwest`. The plan in the future is to also provide a blocking(sync) version and a support for other HTTP clients as well.

## Usage

The async client uses `tokio` and `reqwest`  as dependencies. To use it you would need to setup your `Cargo.toml` to something like this:

```toml
[dependencies]
ddclient = "0.1.0"
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
```

And then use it in your code:

```rust
#[tokio::main]
async fn main() -> Result<(), ApiError> {
    let client = Client::new("my-api-key".to_string());

    let v = client
        .create_voting(vec![
            "Einstein".to_string(),
            "Maxwell".to_string(),
            "Newton".to_string(),
        ])
        .await?;

    Ok(())
}
```

## Features

This client implements all Direct API features.

- Create votings
- Retrieve voting information
- Set voting choices
- Delete votings
- Vote with a ballot
- Unvote
- Get submitted ballot
- Calculate results

## Examples

Feel free to check out the examples in the `examples` directory. These examples cover both regular flow and error handling. Also, for specific errors you can check out the `tests` directory and `APIError` enum.

## Versioning

Each version of the client is tagged and the version is updated accordingly.
To see the list of past versions, run `git tag`.

## Contributing

We love pull requests! Please see the [contribution guidelines](CONTRIBUTING.md).

## License

This library is distributed under the BSD-style license found in the [LICENSE](LICENSE) file.
