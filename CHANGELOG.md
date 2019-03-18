# CHANGELOG

## 0.5.0 (Unreleased)

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
