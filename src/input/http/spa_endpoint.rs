use std::marker::PhantomData;

use poem::{
    Endpoint, Request, Response,
    endpoint::EmbeddedFileEndpoint,
    http::{Method, StatusCode},
};
use rust_embed::RustEmbed;

/// An endpoint that serves static files for a Single Page Application (SPA).
///
/// This endpoint will serve static files normally, but when a file is not found,
/// it will serve the index.html file instead, allowing client-side routing to take over.
pub struct SpaFileEndpoint<E: RustEmbed + Send + Sync> {
    _embed: PhantomData<E>,
}

impl<E: RustEmbed + Send + Sync> SpaFileEndpoint<E> {
    pub fn new() -> Self {
        SpaFileEndpoint {
            _embed: PhantomData,
        }
    }
}

impl<E: RustEmbed + Send + Sync> Default for SpaFileEndpoint<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: RustEmbed + Send + Sync> Endpoint for SpaFileEndpoint<E> {
    type Output = Response;

    async fn call(&self, req: Request) -> Result<Self::Output, poem::Error> {
        if req.method() != Method::GET {
            return Err(StatusCode::METHOD_NOT_ALLOWED.into());
        }

        let path = req.uri().path().trim_start_matches('/');

        if E::get(path).is_some() {
            return EmbeddedFileEndpoint::<E>::new(path).call(req).await;
        }
        EmbeddedFileEndpoint::<E>::new("index.html").call(req).await
    }
}
