use reqwest::Method;
// Import necessary dependencies
use vercel_runtime::{Body, Error, Request, Response, StatusCode};

use crate::crawl_and_search;
use crate::enums::SearchRequest;
use serde_urlencoded::from_str;

// This function handles search requests, performing input validation and returning search results
pub async fn search_handler(request: Request) -> Result<Response<Body>, Error> {
    // Convert the request body to a string
    let search_request: SearchRequest = match *request.method() {
        Method::GET => from_str(request.uri().query().unwrap_or(""))?,
        Method::POST => {
            let body = String::from_utf8(request.body().to_vec())?;
            serde_json::from_str(&body)?
        }
        _ => {
            return Ok(Response::builder()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .body(Body::from("Method not allowed"))
                .unwrap());
        }
    };

    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json");

    // If the query or sitemap_url is empty, return an error with appropriate status code
    if search_request.query.is_empty() {
        return Ok(response
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from("Please specify query"))
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
