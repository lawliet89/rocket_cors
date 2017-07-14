# rocket_cors

[![Build Status](https://travis-ci.org/lawliet89/rocket_cors.svg)](https://travis-ci.org/lawliet89/rocket_cors)
[![Dependency Status](https://dependencyci.com/github/lawliet89/rocket_cors/badge)](https://dependencyci.com/github/lawliet89/rocket_cors)
[![Repository](https://img.shields.io/github/tag/lawliet89/rocket_cors.svg)](https://github.com/lawliet89/rocket_cors)
<!-- [![Crates.io](https://img.shields.io/crates/v/rocket_cors.svg)](https://crates.io/crates/rocket_cors) -->
<!-- [![Documentation](https://docs.rs/rocket_cors/badge.svg)](https://docs.rs/rocket_cors) -->

- Documentation:  stable | [master branch](https://lawliet89.github.io/rocket_cors)

Cross-origin resource sharing (CORS) for [Rocket](https://rocket.rs/) applications

## Requirements

- Nightly Rust
- Rocket > 0.3

### Nightly Rust

Rocket requires nightly Rust. You should probably install Rust with [rustup](https://www.rustup.rs/), then override the code directory to use nightly instead of stable. See
[installation instructions](https://rocket.rs/guide/getting-started/#installing-rust).

In particular, `rocket_cors` is currently targetted for `nightly-2017-07-13`.

### Rocket > 0.3

Rocket > 0.3 is needed. At this moment, `0.3` is not released, and this crate will not be published
to Crates.io until Rocket 0.3 is released to Crates.io.

We currently tie this crate to revision [51a465f2cc88d537079133bcdfec37d029070dcd](https://github.com/SergioBenitez/Rocket/tree/51a465f2cc88d537079133bcdfec37d029070dcd) of Rocket.

## Installation

<!-- Add the following to Cargo.toml:

```toml
rocket_cors = "0.0.6"
``` -->

To use the latest `master` branch, for example:

```toml
rocket_cors = { git = "https://github.com/lawliet89/rocket_cors", branch = "master" }
```

## Reference

- [W3C CORS Recommendation](https://www.w3.org/TR/cors/#resource-processing-model)
