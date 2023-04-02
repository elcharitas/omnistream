// Import necessary dependencies
mod enums;
mod extraction;
pub mod handler;

use enums::{SearchRequest, SearchResult};
use futures::stream::{self, StreamExt};
use select::document::Document;
use select::predicate::{Any, Name};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

fn extract_request(query: SearchRequest) -> SearchRequest {
    match query.sitemap_url {
        Some(url) => SearchRequest {
            query: query.query,
            sitemap_url: Some(url),
            page: query.page,
            per_page: query.per_page,
        },
        None => {
            let mut url = String::from("https://google.com/sitemap.xml");
            let mut real_query = query.query.clone();
            let search_query = query.query.split_whitespace();
            for word in search_query {
                if word.starts_with("site:") {
                    real_query = real_query.replace(word, "");
                    url = word.replace("site:", "https://") + "/sitemap.xml";
                }
            }
            SearchRequest {
                query: real_query,
                sitemap_url: Some(url),
                page: query.page,
                per_page: query.per_page,
            }
        }
    }
}

// This function crawls and searches a website based on the given SearchRequest
pub async fn crawl_and_search(
    request: &SearchRequest,
) -> Result<Vec<SearchResult>, reqwest::Error> {
    let search_request = extract_request(request.clone());
    let sitemap_url = search_request.sitemap_url.clone().unwrap();

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
    let counter = Arc::new(AtomicUsize::new(0));
    let per_page = search_request.per_page.unwrap_or(5) as usize;
    let page = search_request.page.unwrap_or(1) as usize;
    let links: Vec<_> = document
        .find(Name("loc"))
        .map(|n| n.text())
        .skip((page - 1) * per_page)
        .collect();

    // Crawl the links and perform the search in parallel
    let search_query = Arc::new(search_request.query.clone());

    let results = stream::iter(links.into_iter())
        .map(|link| {
            let search_query = Arc::clone(&search_query);
            let counter = Arc::clone(&counter);
            async move {
                if counter.load(Ordering::SeqCst) < per_page {
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
                            counter.fetch_add(1, Ordering::SeqCst);
                            Some(SearchResult {
                                title: title.text(),
                                url: link.clone(),
                                snippet: extraction::extract_snippet(
                                    content.as_str(),
                                    &search_query,
                                ),
                                index: counter.load(Ordering::SeqCst).try_into().unwrap(),
                            })
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        })
        .take_while(|_| async { per_page > counter.load(Ordering::SeqCst) })
        .buffer_unordered(100)
        .filter_map(|result| async move { result })
        .collect::<Vec<SearchResult>>()
        .await;

    Ok(results)
}
