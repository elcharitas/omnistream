use select::document::Document;
use select::node::Node;
use select::predicate::{Attr, Name, Predicate};

// This function extracts a relevant text snippet from the given HTML content and search query
pub fn extract_snippet(html_content: &str, search_query: &str) -> Option<String> {
    let document = Document::from(html_content);
    let search_query_lower = search_query.to_lowercase();

    // Define a custom predicate to match text nodes containing the search query
    let text_contains_query = |node: &Node| {
        if let Some(text) = node.as_text() {
            text.to_lowercase().contains(&search_query_lower)
        } else {
            false
        }
    };

    // Find the first matching text node and extract the snippet
    if let Some(text_node) = document
        .find(Attr("class", "content").descendant(Name("p").child(text_contains_query)))
        .next()
    {
        let snippet = text_node.text().trim().to_string();
        Some(snippet)
    } else {
        None
    }
}
