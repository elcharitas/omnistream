// Import necessary dependencies
mod enums;
mod extraction;
pub mod handler;

use enums::{SearchRequest, SearchResult};
use futures::stream::{self, StreamExt};
use select::document::Document;
use select::predicate::Name;
use std::sync::Arc;

// This function crawls and searches a website based on the given SearchRequest
pub async fn crawl_and_search(search_request: &SearchRequest) -> Vec<SearchResult> {
    // Retrieve the sitemap
    let sitemap = reqwest::get(&search_request.sitemap_url)
        .await
        .expect("Failed to fetch sitemap")
        .text()
        .await
        .expect("Failed to parse sitemap");

    let document = Document::from(sitemap.as_str());

    // Extract links from the sitemap
    let links: Vec<_> = document.find(Name("loc")).map(|n| n.text()).collect();

    // Crawl the links and perform the search in parallel
    let search_request = Arc::new(search_request.clone());
    let results = stream::iter(links.into_iter())
        .map(|link| {
            let search_request = Arc::clone(&search_request);
            async move {
                let content = reqwest::get(&link)
                    .await
                    .expect("Failed to fetch link")
                    .text()
                    .await
                    .expect("Failed to parse link content");

                let doc = Document::from(content.as_str());

                if let Some(title) = doc.find(Name("title")).next() {
                    if title
                        .text()
                        .to_lowercase()
                        .contains(&search_request.query.to_lowercase())
                    {
                        Some(SearchResult {
                            title: title.text(),
                            url: link.clone(),
                            snippet: extraction::extract_snippet(
                                content.as_str(),
                                &search_request.query,
                            ),
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        })
        .buffer_unordered(10)
        .filter_map(|result| async move { result })
        .collect::<Vec<SearchResult>>()
        .await;

    results
}
