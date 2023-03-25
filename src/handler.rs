use vercel_runtime::{Body, Error, Request, Response, StatusCode};

use crate::crawl_and_search;
use crate::enums::SearchRequest;

pub async fn search_handler(request: Request) -> Result<Response<Body>, Error> {
    assert!(request.method() == "POST");
    assert!(request.headers().get("Content-Type").unwrap() == "application/json");
    assert!(&request.body().is_empty());
    let search_request: SearchRequest = serde_json::from_slice(&request.body())?;

    let search_results = crawl_and_search(&search_request).await;
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&search_results).unwrap()))
        .unwrap())
}
