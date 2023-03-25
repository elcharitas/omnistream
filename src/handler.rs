use vercel_runtime::{Body, Error, Request, Response, StatusCode};

use crate::crawl_and_search;
use crate::enums::SearchRequest;

pub async fn search_handler(request: Request) -> Result<Response<Body>, Error> {
    let body = String::from_utf8(request.body().to_vec())?;

    if body.is_empty() {
        return Ok(Response::builder()
            .status(StatusCode::NOT_ACCEPTABLE)
            .header("Content-Type", "application/json")
            .body(Body::from("Please specify query"))
            .unwrap());
    }

    // extract params from body and convert to SearchRequest
    let search_request: SearchRequest = serde_json::from_str(&body).unwrap();

    if search_request.query.is_empty() || search_request.sitemap_url.is_empty() {
        return Ok(Response::builder()
            .status(StatusCode::NOT_ACCEPTABLE)
            .header("Content-Type", "application/json")
            .body(Body::from("Please specify query"))
            .unwrap());
    }

    let search_results = crawl_and_search(&search_request).await;
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&search_results).unwrap()))
        .unwrap())
}
