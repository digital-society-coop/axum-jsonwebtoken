//! [`axum`] extractors for JSON Web Tokens, powered by [`jsonwebtoken`].
//!
//! [`axum`]: https://docs.rs/axum/latest/axum/

#![warn(clippy::pedantic)]
#![cfg_attr(doc, deny(warnings))]

use std::{fmt, sync::Arc};

use axum::{
    extract::{Extension, FromRequest},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use jsonwebtoken::{DecodingKey, TokenData, Validation};
use serde::de::DeserializeOwned;

const AUTHORIZATION_SCHEME: &[u8] = b"Bearer";

type TryBoxFut<'a, T, E> =
    std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send + 'a>>;

/// An extractor for JSON Web Token data.
#[derive(Debug)]
pub struct Jwt<T: DeserializeOwned>(pub TokenData<T>);

impl<T: DeserializeOwned, B: Send> FromRequest<B> for Jwt<T> {
    type Rejection = Error;

    fn from_request<'req, 'ret>(
        req: &'req mut axum::extract::RequestParts<B>,
    ) -> TryBoxFut<'ret, Self, Self::Rejection>
    where
        'req: 'ret,
        Self: 'ret,
    {
        Box::pin(async move {
            let decoding_key: Extension<Arc<DecodingKey>> = FromRequest::from_request(req)
                .await
                .map_err(|_| Error::MissingDecodingKey)?;

            let validation: Extension<Arc<Validation>> = FromRequest::from_request(req)
                .await
                .map_err(|_| Error::MissingValidation)?;

            let token = req
                .headers()
                .and_then(|headers| headers.get("authorization"))
                .and_then(|val| {
                    let slice = val.as_bytes();
                    if slice.starts_with(AUTHORIZATION_SCHEME)
                        && slice.len() > AUTHORIZATION_SCHEME.len()
                        && slice[AUTHORIZATION_SCHEME.len()] == b' '
                    {
                        Some(String::from_utf8_lossy(
                            &slice[AUTHORIZATION_SCHEME.len() + 1..],
                        ))
                    } else {
                        None
                    }
                })
                .ok_or(Error::MissingToken)?;

            let token_data = jsonwebtoken::decode(&token, &decoding_key, &validation)
                .map_err(Error::InvalidToken)?;

            Ok(Self(token_data))
        })
    }
}

/// Errors that can occur when extracting a JWT.
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    MissingDecodingKey,
    MissingValidation,
    MissingToken,
    InvalidToken(jsonwebtoken::errors::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::MissingDecodingKey => write!(
                f,
                "missing `Arc<jsonwebtoken::DecodingKey>` request extension"
            ),
            Self::MissingValidation => write!(
                f,
                "missing `Arc<jsonwebtoken::Validation>` request extension"
            ),
            Self::MissingToken => write!(f, "missing authorization token"),
            // TODO: more info if it's a client error
            Self::InvalidToken(_) => write!(f, "invalid authorization token"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidToken(error) => Some(error),
            _ => None,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::MissingDecodingKey | Error::MissingValidation => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
            }
            Error::MissingToken | Error::InvalidToken(_) => {
                (StatusCode::UNAUTHORIZED, self.to_string()).into_response()
            }
        }
    }
}
