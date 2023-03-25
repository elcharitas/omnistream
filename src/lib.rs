mod enums;
pub mod handler;

use enums::{SearchRequest, SearchResult};
use select::document::Document;
use select::predicate::Name;

pub async fn crawl_and_search(search_request: &SearchRequest) -> Vec<SearchResult> {
    let sitemap = reqwest::get(&search_request.sitemap_url)
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let document = Document::from(sitemap.as_str());

    let links: Vec<_> = document.find(Name("loc")).map(|n| n.text()).collect();

    // Crawl the links and perform the search
    // TODO: Implement parallel crawling
    let mut results = Vec::new();
    for link in links {
        let content = reqwest::get(&link).await.unwrap().text().await.unwrap();
        let doc = Document::from(content.as_str());

        if let Some(title) = doc.find(Name("title")).next() {
            if title
                .text()
                .to_lowercase()
                .contains(&search_request.query.to_lowercase())
            {
                results.push(SearchResult {
                    title: title.text(),
                    url: link.clone(),
                    snippet: None, // TODO: Implement snippet extraction
                });
            }
        }
    }

    results
}
