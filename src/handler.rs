// Import necessary dependencies
use vercel_runtime::{Body, Error, Request, Response, StatusCode};

use crate::crawl_and_search;
use crate::enums::SearchRequest;

// This function handles search requests, performing input validation and returning search results
pub async fn search_handler(request: Request) -> Result<Response<Body>, Error> {
    // Convert the request body to a string
    let body = String::from_utf8(request.body().to_vec())?;
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json");

    // If the request body is empty, return an error with appropriate status code
    if body.is_empty() {
        return Ok(response
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from("Please specify `query` and `sitemap_url`"))
            .unwrap());
    }

    // Try to deserialize the request body into a SearchRequest object, handling any errors
    let search_request: Result<SearchRequest, _> = serde_json::from_str(&body);

    // If deserialization fails, return an error with appropriate status code
    let search_request = match search_request {
        Ok(sr) => sr,
        Err(_) => {
            return Ok(response
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from("Invalid request format"))
                .unwrap());
        }
    };

    // If the query or sitemap_url is empty, return an error with appropriate status code
    if search_request.query.is_empty() || search_request.sitemap_url.is_empty() {
        return Ok(response
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from("Please specify query and sitemap_url"))
            .unwrap());
    }

    // catch the error and return an error with appropriate status code
    let search_results = match crawl_and_search(&search_request).await {
        Ok(sr) => {
            // Return the search results as a JSON object with appropriate status code
            Ok(response
                .body(Body::from(serde_json::to_vec(&sr).unwrap()))
                .unwrap())
        }
        Err(err) => {
            print!("{}", err);
            return Ok(response
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Internal server error"))
                .unwrap());
        }
    };
    search_results
}
