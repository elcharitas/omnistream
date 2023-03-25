// Import necessary dependencies
use vercel_runtime::{Body, Error, Request, Response, StatusCode};

use crate::crawl_and_search;
use crate::enums::SearchRequest;

// This function handles search requests, performing input validation and returning search results
pub async fn search_handler(request: Request) -> Result<Response<Body>, Error> {
    // Convert the request body to a string
    let body = String::from_utf8(request.body().to_vec())?;

    // If the request body is empty, return an error with appropriate status code
    if body.is_empty() {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(Body::from("Please specify query"))
            .unwrap());
    }

    // Try to deserialize the request body into a SearchRequest object, handling any errors
    let search_request: Result<SearchRequest, _> = serde_json::from_str(&body);

    // If deserialization fails, return an error with appropriate status code
    let search_request = match search_request {
        Ok(sr) => sr,
        Err(_) => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header("Content-Type", "application/json")
                .body(Body::from("Invalid request format"))
                .unwrap());
        }
    };

    // If the query or sitemap_url is empty, return an error with appropriate status code
    if search_request.query.is_empty() || search_request.sitemap_url.is_empty() {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "application/json")
            .body(Body::from("Please specify query and sitemap_url"))
            .unwrap());
    }

    // Perform the search using the provided SearchRequest object
    let search_results = crawl_and_search(&search_request).await;

    // Return the search results as a JSON object with appropriate status code
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&search_results).unwrap()))
        .unwrap())
}
