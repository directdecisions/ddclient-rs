# How to Contribute

We welcome patches and contributions to this project. To ensure consistency and quality, we ask that contributors adhere to the few small guidelines.

## Coding Guidelines

- **Code Formatting**: Ensure your code is formatted according to Rust's standard formatting guidelines. You can use `cargo fmt` to automatically format your code.

- **Documentation**: Exported types, constants, variables, and functions should be documented following Rust's documentation guidelines. 

- **Testing**: Changes must be covered with tests. We use `cargo test` to run tests, and all tests must pass. Ensure your new features or fixes include appropriate tests.

- **Versioning**: This Rust client follows [semantic versioning](https://semver.org/). New functionality should be accompanied by an increment to the minor version number.

## Releasing

Releases are made from the `master` branch and should follow these steps:

1. **Update Version Number**: Update the version number in `Cargo.toml` to reflect the new version of the client.

2. **Create a Pull Request**: Make a pull request with the version change and any other relevant updates.

3. **Merge and Release**:
    - Once the pull request has been reviewed and merged, go to the [releases page](https://github.com/directdecisions/ddclient-rs/releases) of the repository.
    - Click "Draft a new release".
    - Set the "Tag version" and "Release title" to the new version of the Rust client, prefixed with `v`, e.g., `v1.2.0`.
    - Publish the release.

