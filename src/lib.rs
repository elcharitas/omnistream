// Import necessary dependencies
mod enums;
mod extraction;
pub mod handler;

use enums::{SearchRequest, SearchResult};
use futures::stream::{self, StreamExt};
use select::document::Document;
use select::predicate::{Any, Name};
use std::sync::Arc;

fn get_sitemap_url(query: SearchRequest) -> String {
    match query.sitemap_url {
        Some(url) => url,
        None => {
            let mut url = String::from("https://google.com/sitemap.xml");
            let query = query.query.split_whitespace();
            for word in query {
                if word.starts_with("site:") {
                    url = word.replace("site:", "");
                }
            }
            url
        }
    }
}

// This function crawls and searches a website based on the given SearchRequest
pub async fn crawl_and_search(
    search_request: &SearchRequest,
) -> Result<Vec<SearchResult>, reqwest::Error> {
    let sitemap_url = get_sitemap_url(search_request.clone());

    // Retrieve the sitemap
    let sitemap_response = reqwest::get(&sitemap_url)
        .await
        .expect("Failed to fetch sitemap");

    let sitemap = sitemap_response
        .text()
        .await
        .expect("Failed to parse sitemap");

    let document = Document::from(sitemap.as_str());

    // Extract links from the sitemap
    let links: Vec<_> = document.find(Name("loc")).map(|n| n.text()).collect();

    // Crawl the links and perform the search in parallel
    let search_query = Arc::new(search_request.query.clone());
    let per_page = search_request.per_page.unwrap_or(5) as usize;
    let page = search_request.page.unwrap_or(1) as usize;

    let results = stream::iter(links.into_iter())
        .skip((page - 1) * per_page)
        .take(per_page)
        .map(|link| {
            let search_query = Arc::clone(&search_query);
            async move {
                let content = reqwest::get(&link)
                    .await
                    .expect("Failed to fetch link")
                    .text()
                    .await
                    .expect("Failed to parse link content");

                let search_doc = Document::from(content.as_str());

                if let Some(title) = search_doc.find(Name("title")).next() {
                    let has_query = search_doc.find(Any).any(|element| {
                        element
                            .text()
                            .to_lowercase()
                            .contains(&search_query.to_lowercase())
                    });
                    if has_query {
                        Some(SearchResult {
                            title: title.text(),
                            url: link.clone(),
                            snippet: extraction::extract_snippet(content.as_str(), &search_query),
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

    Ok(results)
}
