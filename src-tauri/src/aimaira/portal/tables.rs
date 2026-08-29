//! The generic readers: any `<table>` the portal renders, the label/value pairs
//! of the profile page, and the header matching every column lookup goes
//! through.

use std::collections::{HashMap, HashSet};

use scraper::{ElementRef, Html};

use super::html::element_text;
use super::model::{PortalField, PortalTable};
use super::selectors;
use crate::aimaira::html_to_text;

pub(super) fn parse_semantic_tables(document: &Html) -> (Vec<String>, Vec<PortalTable>) {
    let mut context = Vec::<String>::new();
    let mut headings = Vec::new();
    let mut tables = Vec::new();

    for element in document.select(&selectors::HEADING_OR_TABLE) {
        let name = element.value().name();
        if let Some(level) = heading_level(name) {
            let text = element_text(element);
            if text.is_empty() {
                continue;
            }
            context.truncate(level.saturating_sub(1));
            while context.len() < level.saturating_sub(1) {
                context.push(String::new());
            }
            context.push(text.clone());
            headings.push(text);
            continue;
        }

        if let Some(table) = parse_table(element, &context) {
            tables.push(table);
        }
    }

    (headings, tables)
}

fn parse_table(table: ElementRef<'_>, context: &[String]) -> Option<PortalTable> {
    let caption = table
        .select(&selectors::TABLE_CAPTION)
        .map(element_text)
        .find(|text| !text.is_empty());
    let mut headers = Vec::new();
    let mut rows = Vec::new();

    for row in table.select(&selectors::TABLE_ROW) {
        let cells = row
            .select(&selectors::TABLE_CELL)
            .map(element_text)
            .collect::<Vec<_>>();
        if cells.iter().all(String::is_empty) {
            continue;
        }

        let is_header_row = row.select(&selectors::TABLE_DATA_CELL).next().is_none();
        if headers.is_empty() && is_header_row {
            headers = cells;
        } else {
            rows.push(cells);
        }
    }

    if headers.is_empty() && rows.is_empty() {
        return None;
    }

    Some(PortalTable {
        context: context
            .iter()
            .filter(|heading| !heading.is_empty())
            .cloned()
            .collect(),
        caption,
        headers,
        rows,
    })
}

pub(super) fn parse_fields(document: &Html) -> Vec<PortalField> {
    let mut fields = Vec::new();
    let mut seen = HashSet::new();

    for list in document.select(&selectors::DEFINITION_LIST) {
        let mut label = None;
        for item in list.select(&selectors::DEFINITION_ITEM) {
            match item.value().name() {
                "dt" => label = Some(element_text(item)),
                "dd" => {
                    if let Some(label) = label.take() {
                        push_field(&mut fields, &mut seen, label, element_text(item));
                    }
                }
                _ => {}
            }
        }
    }

    let controls = collect_form_controls(document);
    for label in document.select(&selectors::LABELLED_CONTROL) {
        let Some(control_id) = label.value().attr("for") else {
            continue;
        };
        let Some(value) = controls.get(control_id) else {
            continue;
        };
        push_field(&mut fields, &mut seen, element_text(label), value.clone());
    }

    fields
}

fn collect_form_controls(document: &Html) -> HashMap<String, String> {
    let mut controls = HashMap::new();

    for control in document.select(&selectors::FORM_CONTROL) {
        let Some(id) = control.value().attr("id") else {
            continue;
        };
        let value = match control.value().name() {
            "input" => {
                let input_type = control.value().attr("type").unwrap_or("text");
                if matches!(
                    input_type,
                    "hidden"
                        | "password"
                        | "file"
                        | "checkbox"
                        | "radio"
                        | "button"
                        | "submit"
                        | "reset"
                        | "image"
                ) {
                    continue;
                }
                control.value().attr("value").unwrap_or_default().to_owned()
            }
            "select" => control
                .select(&selectors::SELECTED_OPTION)
                .next()
                .or_else(|| control.select(&selectors::OPTION).next())
                .map(element_text)
                .unwrap_or_default(),
            "textarea" => element_text(control),
            _ => continue,
        };

        controls.insert(id.to_owned(), html_to_text(&value));
    }

    controls
}

fn push_field(
    fields: &mut Vec<PortalField>,
    seen: &mut HashSet<(String, String)>,
    label: String,
    value: String,
) {
    let label = html_to_text(&label);
    let value = html_to_text(&value);
    if label.is_empty() || value.is_empty() || !seen.insert((label.clone(), value.clone())) {
        return;
    }
    fields.push(PortalField { label, value });
}

fn heading_level(name: &str) -> Option<usize> {
    name.strip_prefix('h')?
        .parse::<usize>()
        .ok()
        .filter(|level| (1..=6).contains(level))
}

pub(super) fn find_header(headers: &[String], candidates: &[&str]) -> Option<usize> {
    headers.iter().position(|header| {
        let normalized = normalize_header(header);
        candidates
            .iter()
            .any(|candidate| normalized.contains(candidate))
    })
}

pub(super) fn normalize_header(value: &str) -> String {
    value
        .to_lowercase()
        .replace([' ', '-', '_', '.'], "")
        .replace(['é', 'è', 'ê'], "e")
}
