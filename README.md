# rocket_cors

[![Continuous integration](https://github.com/lawliet89/rocket_cors/actions/workflows/rust.yml/badge.svg)](https://github.com/lawliet89/rocket_cors/actions/workflows/rust.yml)
[![Repository](https://img.shields.io/github/tag/lawliet89/rocket_cors.svg)](https://github.com/lawliet89/rocket_cors)
[![Crates.io](https://img.shields.io/crates/v/rocket_cors.svg)](https://crates.io/crates/rocket_cors)

- Documentation: [master branch](https://lawliet89.github.io/rocket_cors) | [stable](https://docs.rs/rocket_cors)

Cross-origin resource sharing (CORS) for [Rocket](https://rocket.rs/) applications

## Requirements

- Rocket >= 0.4

If you are using Rocket 0.3, use the `0.3.0` version of this crate.

## Installation

Add the following to Cargo.toml:

```toml
rocket_cors = "0.6.0"
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
