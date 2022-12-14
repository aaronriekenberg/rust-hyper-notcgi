use std::{borrow::Cow, collections::HashMap, path::PathBuf};

use anyhow::Context;

use async_trait::async_trait;

use hyper::{http::Method, Body, Response};

use crate::handlers::{utils::build_status_code_response, HttpRequest, RequestHandler};

pub struct RouteInfo {
    pub method: &'static Method,
    pub path_suffix: PathBuf,
    pub handler: Box<dyn RequestHandler>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct RouteKey<'a> {
    method: &'a Method,
    path: Cow<'a, str>,
}

impl<'a> From<&'a HttpRequest> for RouteKey<'a> {
    fn from(http_request: &'a HttpRequest) -> Self {
        Self {
            method: http_request.hyper_request().method(),
            path: Cow::from(http_request.hyper_request().uri().path()),
        }
    }
}

pub struct Router {
    route_key_to_handler: HashMap<RouteKey<'static>, Box<dyn RequestHandler>>,
}

impl Router {
    pub fn new(routes: Vec<RouteInfo>) -> anyhow::Result<Self> {
        let mut router = Self {
            route_key_to_handler: HashMap::with_capacity(routes.len()),
        };

        let context_configuration = crate::config::instance().context_configuration();

        for route in routes {
            let uri_pathbuf =
                PathBuf::from(context_configuration.context()).join(route.path_suffix);

            let path = uri_pathbuf
                .to_str()
                .with_context(|| {
                    format!(
                        "Router::new error: uri_pathbuf.to_str error uri_pathbuf = '{:?}'",
                        uri_pathbuf,
                    )
                })?
                .to_owned();

            let key = RouteKey {
                method: route.method,
                path: Cow::from(path),
            };

            if router
                .route_key_to_handler
                .insert(key.clone(), route.handler)
                .is_some()
            {
                anyhow::bail!("Router::new error: collision in router key = {:?}", key);
            }
        }
        Ok(router)
    }
}

#[async_trait]
impl RequestHandler for Router {
    async fn handle(&self, request: &HttpRequest) -> Response<Body> {
        let handler_option = self.route_key_to_handler.get(&RouteKey::from(request));

        match handler_option {
            None => build_status_code_response(hyper::http::StatusCode::NOT_FOUND),
            Some(handler) => handler.handle(&request).await,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_route_key_equality() {
        assert_eq!(
            RouteKey {
                method: &Method::GET,
                path: Cow::Borrowed("/test"),
            },
            RouteKey {
                method: &Method::GET,
                path: Cow::Owned("/test".to_owned()),
            }
        );

        assert_ne!(
            RouteKey {
                method: &Method::GET,
                path: Cow::Borrowed("/test"),
            },
            RouteKey {
                method: &Method::PUT,
                path: Cow::Owned("/test".to_owned()),
            }
        );

        assert_ne!(
            RouteKey {
                method: &Method::GET,
                path: Cow::Borrowed("/nottest"),
            },
            RouteKey {
                method: &Method::GET,
                path: Cow::Owned("/test".to_owned()),
            }
        );
    }

    #[test]
    fn test_route_key_hash() {
        use std::{
            collections::hash_map::DefaultHasher,
            hash::{Hash, Hasher},
        };

        let key1 = RouteKey {
            method: &Method::GET,
            path: Cow::Borrowed("/test"),
        };

        let key2 = RouteKey {
            method: &Method::GET,
            path: Cow::Owned("/test".to_owned()),
        };

        let mut s = DefaultHasher::new();
        key1.hash(&mut s);
        let key1_hash = s.finish();

        let mut s = DefaultHasher::new();
        key2.hash(&mut s);
        let key2_hash = s.finish();

        assert_eq!(key1_hash, key2_hash);
    }
}
