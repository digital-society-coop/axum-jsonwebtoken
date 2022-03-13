# `axum-jsonwebtoken`

[`axum`] extractors for JSON Web Tokens, powered by [jsonwebtoken].

[`axum`]: https://github.com/tokio-rs/axum#readme
[jsonwebtoken]: https://github.com/Keats/jsonwebtoken#readme

## Usage

1. Install `axum-jsonwebtoken`:

  ```sh
  cargo add axum-jsonwebtoken
  ```

2. Define a struct for your claims, deriving [`serde::Deserialize`]:

  ```rust
  #[derive(serde::Deserialize)]
  struct Claims {
      sub: String,
      company: String,
  }
  ```

3. Set your desired [`jsonwebtoken::DecodingKey`] and [`jsonwebtoken::Validation`] as [request extensions]:

  ```rust
  use axum::extract::Extension;

  let decoding_key: jsonwebtoken::DecodingKey = todo!();
  let validation: jsonwebtoken::Validation = todo!();

  let app = axum::Router::new()
      /* ... routes ... */
      .layer(Extension(Arc::new(decoding_key)))
      .layer(Extension(Arc::new(validation)));
  ```

4. Use `axum_jsonwebtoken::Jwt` to extract the claims in your `axum` handlers:

  ```rust
  use axum_jsonwebtoken::Jwt;

  async fn identify(Jwt(claims): axum_jsonwebtoken::Jwt<Claims>) {
      /* ... */
  }
  ```

[`serde::Deserialize`]: https://docs.rs/serde/latest/serde/trait.Deserialize.html
[`jsonwebtoken::DecodingKey`]: https://docs.rs/jsonwebtoken/latest/jsonwebtoken/struct.DecodingKey.html
[`jsonwebtoken::Validation`]: https://docs.rs/jsonwebtoken/latest/jsonwebtoken/struct.Validation.html
[request extensions]: https://docs.rs/axum/latest/axum/#using-request-extensions

## Caveats and future work

- For now, JWT decoding configuration must be static (e.g. no support for fetching JWKs on-demand).
  This could be implemented by introducing a [`Layer`] to handle the additional configuration (and perhaps take over the existing configuration as well).

- Similarly, tokens MUST be in the `authorization` header and MUST have a `Bearer ` prefix.
  This should become configurable in future.

- Some error information is swallowed by default.
  You can use the techniques documented [here] to apply your own error handling.
  In future this may be simplified.

- To simplify this initial implementation the library currently depends directly on `axum`, rather than `axum-core`.
  This may be a maintenance hazard and will be fixed in future.

[here]: https://docs.rs/axum/0.4.8/axum/extract/index.html#customizing-extractor-responses
