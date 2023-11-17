# CHANGELOG

## 0.6.0-alpha3 (2023-11-17)

## What's Changed
* Update to latest master Rocket version (#89) by @j03-dev in https://github.com/lawliet89/rocket_cors/pull/114

## New Contributors
* @j03-dev made their first contribution in https://github.com/lawliet89/rocket_cors/pull/114

**Full Changelog**: https://github.com/lawliet89/rocket_cors/compare/v0.6.0-alpha2...v0.6.0-alpha3

## 0.6.0-alpha2 (2022-01-13)

## What's Changed
* Use async version from rocket's master branch by @HenningHolmDE in https://github.com/lawliet89/rocket_cors/pull/81
* fix: Update to latest master Rocket version by @DusterTheFirst in https://github.com/lawliet89/rocket_cors/pull/85
* Update to latest master Rocket version by @thanadolps in https://github.com/lawliet89/rocket_cors/pull/89
* Upgrade to GitHub-native Dependabot by @dependabot-preview in https://github.com/lawliet89/rocket_cors/pull/90
* Update to latest Rocket master by @ELD in https://github.com/lawliet89/rocket_cors/pull/91
* Resolve Tokio Dependency conflicy by @magpie-engineering in https://github.com/lawliet89/rocket_cors/pull/92
* Update to Rocket 0.5-rc.1 by @ELD in https://github.com/lawliet89/rocket_cors/pull/93
* Update lib.rs and README for nightly req and version by @jtroo in https://github.com/lawliet89/rocket_cors/pull/95
* Responder lifetime cannot be infered by @mrene in https://github.com/lawliet89/rocket_cors/pull/97
* Fix documentation typos by @deneiruy in https://github.com/lawliet89/rocket_cors/pull/98
* Fix rustdoc lint drift by @ELD in https://github.com/lawliet89/rocket_cors/pull/101
* Drop body from response to preflight request by @KOBA789 in https://github.com/lawliet89/rocket_cors/pull/100
* docs: fix ci badge by @torkleyy in https://github.com/lawliet89/rocket_cors/pull/104
* feat: update rust edition from 2018 to 2021 by @somehowchris in https://github.com/lawliet89/rocket_cors/pull/105

## New Contributors
* @HenningHolmDE made their first contribution in https://github.com/lawliet89/rocket_cors/pull/81
* @DusterTheFirst made their first contribution in https://github.com/lawliet89/rocket_cors/pull/85
* @thanadolps made their first contribution in https://github.com/lawliet89/rocket_cors/pull/89
* @magpie-engineering made their first contribution in https://github.com/lawliet89/rocket_cors/pull/92
* @jtroo made their first contribution in https://github.com/lawliet89/rocket_cors/pull/95
* @mrene made their first contribution in https://github.com/lawliet89/rocket_cors/pull/97
* @deneiruy made their first contribution in https://github.com/lawliet89/rocket_cors/pull/98
* @KOBA789 made their first contribution in https://github.com/lawliet89/rocket_cors/pull/100
* @torkleyy made their first contribution in https://github.com/lawliet89/rocket_cors/pull/104
* @somehowchris made their first contribution in https://github.com/lawliet89/rocket_cors/pull/105

**Full Changelog**: https://github.com/lawliet89/rocket_cors/compare/v0.5.2...v0.6.0-alpha2

## 0.5.2 (2020-03-18)

### Improvements

- Add a builder methods for `CorsOptions` (#75)

## 0.5.1 (2019-11-13)

There are no new features.

- Fix build issues with Rocket 0.4.2
- Fix clippy lints with latest nightly

## <a name="0.5.0"></a>0.5.0 (2019-05-27)

There is no change since `0.5.0-beta1`.

### Breaking Changes

- The [`Cors`](https://lawliet89.github.io/rocket_cors/rocket_cors/struct.Cors.html) struct can no
    longer be constructed. Instead, you will now construct the options for Cors directly or through
    deserialization using the
    [`CorsOptions`](https://lawliet89.github.io/rocket_cors/rocket_cors/struct.CorsOptions.html)
    struct. Then, you can construct `Cors` for use in Fairings or manual responses using the
    [`CorsOptions::to_cors`](https://lawliet89.github.io/rocket_cors/rocket_cors/struct.CorsOptions.html#method.to_cors)
    method.
- The
    [`AllowedOrigins`](https://lawliet89.github.io/rocket_cors/rocket_cors/type.AllowedOrigins.html)
    type has been modified. It is now a typedef of `AllOrSome<Origins>` where
    [`Origins`](https://lawliet89.github.io/rocket_cors/rocket_cors/struct.Origins.html) is now
    a struct supporting exact matches or regex matches.

### Migrating existing Code

- Existing use of
    [`AllowedOrigins::some`](https://docs.rs/rocket_cors/0.4.0/rocket_cors/type.AllowedOrigins.html#method.some)
    to create exact matches can be replaced simply with
    [`AllowedOrigins::some_exact`](https://lawliet89.github.io/rocket_cors/rocket_cors/type.AllowedOrigins.html#method.some_exact)
    instead.
- Replace all construction of `Cors` struct with `CorsOptions` instead. Then, you can create the
    `Cors` struct for use in Fairings using the
    [`CorsOptions::to_cors`](https://lawliet89.github.io/rocket_cors/rocket_cors/struct.CorsOptions.html#method.to_cors)
    method

    ```diff
    -fn main() {
    +fn main() -> Result<(), Error> {
        let (allowed_origins, failed_origins) = AllowedOrigins::some(&["https://www.acme.com"]);
        assert!(failed_origins.is_empty());

        // You can also deserialize this
    -    let options = rocket_cors::Cors {
    +    let cors = rocket_cors::CorsOptions {
            allowed_origins: allowed_origins,
            allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
            allowed_headers: AllowedHeaders::some(&["Authorization", "Accept"]),
            allow_credentials: true,
            ..Default::default()
    -    };
    +    }
    +    .to_cors()?;

        rocket::ignite()
            .mount("/", routes![cors])
    -        .attach(options)
    +        .attach(cors)
            .launch();
    +
    +    Ok(())
    }
    ```
