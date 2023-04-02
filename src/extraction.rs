use select::document::Document;
use select::node::Node;
use select::predicate::Any;

use crate::enums::SearchRequest;

pub fn extract_request(query: SearchRequest) -> SearchRequest {
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

// This function extracts a relevant text snippet from the given HTML content and search query
pub fn extract_snippet(html_content: &str, search_query: &str) -> Option<String> {
    let document = Document::from(html_content);

    // Define a custom predicate to match text nodes containing the search query
    let node_has_query = |node: &Node| {
        if let Some(text) = node.as_text() {
            text.to_lowercase().contains(&search_query.to_lowercase())
        } else {
            false
        }
    };

    // Find the first matching text node where search_query is found and extract the snippet
    if let Some(text_node) = document.find(Any).filter(node_has_query).next() {
        let snippet = text_node.as_text()?.trim().to_string();
        Some(snippet)
    } else {
        None
    }
}
