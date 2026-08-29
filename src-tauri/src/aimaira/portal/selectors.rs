//! Every CSS selector the portal parsers run, compiled once.
//!
//! Two of the parsers used to rebuild theirs on each call: the grades page
//! compiled four selectors per course tile — a real page carries three years of
//! two blocks of some fifteen courses, so several hundred compilations per load
//! — and every table compiled four more. Holding them here compiles each one
//! once for the life of the process, and takes the `Selector::parse(..)` out of
//! the parsers so a reader of `grades.rs` sees what is being matched, not how
//! the matcher is built.

use std::sync::LazyLock;

use scraper::Selector;

/// Every selector below is a constant this file owns, so a parse failure is a
/// typo, not a runtime condition a call site could recover from. Failing at
/// first use keeps the call sites free of a `Result` none of them could answer.
macro_rules! selector {
    ($(#[$attribute:meta])* $name:ident = $css:literal) => {
        $(#[$attribute])*
        pub(super) static $name: LazyLock<Selector> = LazyLock::new(|| {
            Selector::parse($css).expect(concat!("invalid portal selector: ", $css))
        });
    };
}

// Shared structure.
selector!(ANCHOR = "a[href]");
selector!(SCRIPT = "script");
selector!(HEADING = "h1, h2, h3, h4, h5, h6");
selector!(SMALL = "small");

// Page title, in the order `parse_portal_document` tries them.
selector!(MAIN_HEADING = "main h1");
selector!(MAIN_SUBHEADING = "main h2");
selector!(DOCUMENT_HEADING = "h1");
selector!(DOCUMENT_TITLE = "title");

// Year → block accordions, shared by the grades and the absences pages.
selector!(PERIOD_PANEL = "#accordion-periode > .panel");
selector!(BLOCK_PANEL = ".panel-group > .panel");
selector!(PANEL_HEADING = ".panel-heading");
selector!(PANEL_TITLE = ".panel-title");

// Grade course tiles.
selector!(TILES_LIST = ".tiles-list");
selector!(TILE = ".tile");
selector!(TILE_TITLE = ".panel-heading strong");
selector!(TILE_ANCHOR = ".panel-heading a[href]");

// Definition lists: profile fields, and the evaluations of a course tile.
selector!(DEFINITION_LIST = "dl");
selector!(DEFINITION_ITEM = "dt, dd");

// Tables.
selector!(TABLE = "table");
selector!(HEADING_OR_TABLE = "h1, h2, h3, h4, h5, h6, table");
selector!(TABLE_CAPTION = "caption");
selector!(TABLE_ROW = "tr");
selector!(TABLE_CELL = "th, td");
selector!(TABLE_DATA_CELL = "td");
selector!(TABLE_HEAD_CELL = "thead th");
selector!(TABLE_BODY_ROW = "tbody tr");

// Form controls read as profile fields.
selector!(FORM_CONTROL = "input[id], select[id], textarea[id]");
selector!(SELECTED_OPTION = "option[selected]");
selector!(OPTION = "option");
selector!(LABELLED_CONTROL = "label[for]");
selector!(LABEL = "label");

// Questionnaire list rows.
selector!(QUESTIONNAIRE_RESPONSE_ANCHOR = "a[href*='/Questionnaire/Reponse']");
selector!(QUESTIONNAIRE_TITLE = "strong");
selector!(QUESTIONNAIRE_CONTEXT = ".col-lg-6");
selector!(QUESTIONNAIRE_DEADLINE = ".col-lg-3");
