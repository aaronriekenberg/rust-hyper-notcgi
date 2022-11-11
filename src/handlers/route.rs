use std::{collections::HashMap, path::PathBuf};

use anyhow::Context;

use async_trait::async_trait;

use hyper::{Body, Response};

use crate::handlers::{utils::build_status_code_response, HttpRequest, RequestHandler};

pub type PathSuffixAndHandler = (PathBuf, Box<dyn RequestHandler>);

pub struct Router {
    uri_to_request_handler: HashMap<String, Box<dyn RequestHandler>>,
}

impl Router {
    pub fn new(routes: Vec<PathSuffixAndHandler>) -> anyhow::Result<Self> {
        let mut router = Self {
            uri_to_request_handler: HashMap::with_capacity(routes.len()),
        };

        let context_configuration = crate::config::instance().context_configuration();

        for (path_suffix, handler) in routes {
            let uri_pathbuf = PathBuf::from(context_configuration.context()).join(path_suffix);

            let uri = uri_pathbuf.to_str().with_context(|| {
                format!(
                    "Router::new error: route path contains invalid UTF-8 uri_pathbuf = '{:?}'",
                    uri_pathbuf,
                )
            })?;

            if router
                .uri_to_request_handler
                .insert(uri.to_owned(), handler)
                .is_some()
            {
                anyhow::bail!("Router::new error: collision in router uri '{}'", uri);
            }
        }
        Ok(router)
    }
}

#[async_trait]
impl RequestHandler for Router {
    async fn handle(&self, request: HttpRequest) -> Response<Body> {
        match self
            .uri_to_request_handler
            .get(request.hyper_request().uri().path())
        {
            Some(handler) => handler.handle(request).await,
            None => build_status_code_response(hyper::http::StatusCode::NOT_FOUND),
        }
    }
}
