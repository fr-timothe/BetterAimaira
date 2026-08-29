//! Small readers shared by every portal parser: text out of an element, a URL
//! the portal itself served, and the JSON the portal inlines in a `<script>`.

use reqwest::Url;
use scraper::{ElementRef, Html, Selector};
use serde_json::Value;

use super::selectors;
use crate::aimaira::html_to_text;

/// Returns the text of the first candidate that matches, trying them in the order given.
///
/// One selector list would not do: `Html::select` walks the document, not the list, and
/// `<title>` sits in `<head>` ahead of every heading in `<body>`, so the fallback would
/// always answer before the page heading it was meant to back up.
pub(super) fn select_first_text(document: &Html, candidates: &[&Selector]) -> Option<String> {
    candidates.iter().find_map(|candidate| {
        document
            .select(candidate)
            .map(element_text)
            .find(|text| !text.is_empty())
    })
}

pub(super) fn element_text(element: ElementRef<'_>) -> String {
    element
        .text()
        .flat_map(str::split_whitespace)
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) fn same_origin_url(portal_url: &Url, href: Option<&str>) -> Option<Url> {
    let url = portal_url.join(href?).ok()?;
    (url.origin() == portal_url.origin()).then_some(url)
}

pub(super) fn request_path(url: &Url) -> String {
    match url.query() {
        Some(query) => format!("{}?{query}", url.path()),
        None => url.path().to_owned(),
    }
}

pub(super) fn extract_script_json(document: &Html, variable: &str) -> Option<Value> {
    let declarations = [
        format!("var {variable}"),
        format!("let {variable}"),
        format!("const {variable}"),
    ];

    for script in document.select(&selectors::SCRIPT) {
        let source = script.text().collect::<String>();
        let Some(declaration_start) = declarations
            .iter()
            .filter_map(|declaration| source.find(declaration))
            .min()
        else {
            continue;
        };
        let assignment = source[declaration_start..].find('=')? + declaration_start + 1;
        let value_source = source[assignment..].trim_start();
        let mut values = serde_json::Deserializer::from_str(value_source).into_iter::<Value>();
        if let Some(Ok(value)) = values.next() {
            return Some(value);
        }
    }
    None
}

pub(super) fn json_scalar_text(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(html_to_text(value)),
        Value::Number(value) => Some(value.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}
