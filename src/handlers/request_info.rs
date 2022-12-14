use std::{collections::BTreeMap, path::PathBuf};

use async_trait::async_trait;

use hyper::{http::Method, http::Version, Body, Response};

use serde::Serialize;

use crate::handlers::{route::RouteInfo, utils::build_json_response, HttpRequest, RequestHandler};

#[derive(Debug, Serialize)]
struct RequestInfoResponse<'a> {
    connection_id: u64,
    request_id: u64,
    method: &'a str,
    version: &'a str,
    request_uri_path: &'a str,
    http_headers: BTreeMap<&'a str, &'a str>,
}

struct RequestInfoHandler {}

impl RequestInfoHandler {
    fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl RequestHandler for RequestInfoHandler {
    async fn handle(&self, request: &HttpRequest) -> Response<Body> {
        let hyper_request = request.hyper_request();

        let version = match hyper_request.version() {
            Version::HTTP_09 => "HTTP/0.9",
            Version::HTTP_10 => "HTTP/1.0",
            Version::HTTP_11 => "HTTP/1.1",
            Version::HTTP_2 => "HTTP/2.0",
            Version::HTTP_3 => "HTTP/3.0",
            _ => "[Unknown]",
        };

        let response = RequestInfoResponse {
            connection_id: *request.connection_id(),
            request_id: *request.request_id(),
            method: hyper_request.method().as_str(),
            version,
            request_uri_path: hyper_request.uri().path(),
            http_headers: hyper_request
                .headers()
                .iter()
                .map(|(key, value)| (key.as_str(), value.to_str().unwrap_or("[Unknown]")))
                .collect(),
        };

        build_json_response(response)
    }
}

pub fn create_routes() -> Vec<RouteInfo> {
    vec![RouteInfo {
        method: &Method::GET,
        path_suffix: PathBuf::from("request_info"),
        handler: Box::new(RequestInfoHandler::new()),
    }]
}
