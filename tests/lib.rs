use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::Extension,
    http::{Request, StatusCode},
    response::Response,
};
use axum_jsonwebtoken::Jwt;
use jsonwebtoken::{DecodingKey, EncodingKey, Validation};
use tower::ServiceExt;

#[tokio::test]
async fn basic() {
    #[derive(serde::Deserialize, serde::Serialize)]
    struct Claims {
        exp: u64,
        hello: String,
    }

    let decoding_key = DecodingKey::from_secret(b"secret");
    let validation = Validation::default();
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &Claims {
            exp: jsonwebtoken::get_current_timestamp() + 300,
            hello: "world".to_string(),
        },
        &EncodingKey::from_secret(b"secret"),
    )
    .unwrap();

    let app = axum::Router::new()
        .route(
            "/",
            axum::routing::get(|Jwt(token): Jwt<Claims>| async move { token.claims.hello }),
        )
        .layer(Extension(Arc::new(decoding_key)))
        .layer(Extension(Arc::new(validation)));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/")
                .header("authorization", format!("Bearer {token}"))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = check_response(StatusCode::OK, response).await;
    assert_eq!(&body[..], b"world");
}

async fn check_response(expected_status: StatusCode, response: Response) -> Bytes {
    let actual_status = response.status();
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

    if actual_status == expected_status {
        body
    } else {
        panic!(
            "unexpected response: expected {expected_status} but found {actual_status}\nbody: {}",
            String::from_utf8_lossy(&body[..])
        );
    }
}
