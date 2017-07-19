# rocket_cors

[![Build Status](https://travis-ci.org/lawliet89/rocket_cors.svg)](https://travis-ci.org/lawliet89/rocket_cors)
[![Dependency Status](https://dependencyci.com/github/lawliet89/rocket_cors/badge)](https://dependencyci.com/github/lawliet89/rocket_cors)
[![Repository](https://img.shields.io/github/tag/lawliet89/rocket_cors.svg)](https://github.com/lawliet89/rocket_cors)
[![Crates.io](https://img.shields.io/crates/v/rocket_cors.svg)](https://crates.io/crates/rocket_cors)

- Documentation: [master branch](https://lawliet89.github.io/rocket_cors)

Cross-origin resource sharing (CORS) for [Rocket](https://rocket.rs/) applications

## Requirements

- Nightly Rust
- Rocket >= 0.3

### Nightly Rust

Rocket requires nightly Rust. You should probably install Rust with
[rustup](https://www.rustup.rs/), then override the code directory to use nightly instead of stable.
See
[installation instructions](https://rocket.rs/guide/getting-started/#installing-rust).

In particular, `rocket_cors` is currently targetted for `nightly-2017-07-13`. Newer nightlies
might work, but it's not guaranteed.

## Installation

Add the following to Cargo.toml:

```toml
rocket_cors = "0.1.3"
```

To use the latest `master` branch, for example:

```toml
rocket_cors = { git = "https://github.com/lawliet89/rocket_cors", branch = "master" }
```

## Reference

- [W3C CORS Recommendation](https://www.w3.org/TR/cors/#resource-processing-model)
