# rocket_cors

[![Build Status](https://travis-ci.org/lawliet89/rocket_cors.svg)](https://travis-ci.org/lawliet89/rocket_cors)
[![Dependency Status](https://dependencyci.com/github/lawliet89/rocket_cors/badge)](https://dependencyci.com/github/lawliet89/rocket_cors)
[![Repository](https://img.shields.io/github/tag/lawliet89/rocket_cors.svg)](https://github.com/lawliet89/rocket_cors)
[![Crates.io](https://img.shields.io/crates/v/rocket_cors.svg)](https://crates.io/crates/rocket_cors)

- Documentation: [master branch](https://lawliet89.github.io/rocket_cors) | [stable](https://docs.rs/rocket_cors)

Cross-origin resource sharing (CORS) for [Rocket](https://rocket.rs/) applications

## Requirements

- Nightly Rust
- Rocket >= 0.4

If you are using Rocket 0.3, use the `0.3.0` version of this crate.

### Nightly Rust

Rocket requires nightly Rust. You should probably install Rust with
[rustup](https://www.rustup.rs/), then override the code directory to use nightly instead of stable.
See
[installation instructions](https://rocket.rs/guide/getting-started/#installing-rust).

In particular, `rocket_cors` is currently targetted for the latest `nightly`. Older nightlies might
work, but they are subject to the minimum that Rocket sets.

## Installation

Add the following to Cargo.toml:

```toml
rocket_cors = "0.4.0-rc.1"
```

To use the latest `master` branch, for example:

```toml
rocket_cors = { git = "https://github.com/lawliet89/rocket_cors", branch = "master" }
```

## Reference

- [W3C CORS Recommendation](https://www.w3.org/TR/cors/#resource-processing-model)

## License

`rocket_cors` is licensed under either of the following, at your option:

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
