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

In particular, `rocket_cors` is currently targetted for the latest `nightly`. Older nightlies might
work, but they are subject to the minimum that
[Rocket](https://github.com/SergioBenitez/Rocket/blob/master/codegen/build.rs) sets.

## Installation

Add the following to Cargo.toml:

```toml
rocket_cors = "0.2.1"
```

To use the latest `master` branch, for example:

```toml
rocket_cors = { git = "https://github.com/lawliet89/rocket_cors", branch = "master" }
```

## Reference

- [W3C CORS Recommendation](https://www.w3.org/TR/cors/#resource-processing-model)
