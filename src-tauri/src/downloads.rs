//! Where a downloaded portal PDF lands on disk.
//!
//! The webview cannot save it on its own: an `<a download>` pointing at a
//! `blob:` URL is ignored outright by WKWebView and the Android WebView, and
//! WebView2 cancels the transfer the moment the object URL is revoked — the
//! button finished loading and nothing appeared. Writing the bytes from Rust
//! is the one path that behaves the same on every platform.

use std::fs;
use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};

use crate::error::CommandError;

/// Stop before the filesystem does: a portal label can be long, and a name that
/// overruns the platform limit fails the write instead of the download.
const MAX_STEM_CHARS: usize = 120;

/// How many `name (2).pdf` variants to try before giving up on a free name.
const MAX_NAME_ATTEMPTS: u32 = 100;

/// Characters that would either steer the write out of the folder or be
/// rejected outright by NTFS. Replaced by a dash, never dropped, so two
/// documents cannot collapse onto the same name.
const SEPARATORS: &str = r#"/\:<>"|?*"#;

/// Writes the document next to the user's other downloads and answers with the
/// path it took, so the caller can name it and offer to open it.
pub fn save_document(
    app: &AppHandle,
    filename: &str,
    bytes: &[u8],
) -> Result<PathBuf, CommandError> {
    let directory = download_directory(app)?;
    fs::create_dir_all(&directory).map_err(|_| CommandError::new("document_save_failed"))?;
    let target = free_path(&directory, &safe_filename(filename));
    fs::write(&target, bytes).map_err(|_| CommandError::new("document_save_failed"))?;
    Ok(target)
}

/// The system download folder when there is one. Android and iOS have no shared
/// downloads directory the app may write to, so they fall back to the app's own
/// documents folder, which the file picker still lists.
fn download_directory(app: &AppHandle) -> Result<PathBuf, CommandError> {
    let resolver = app.path();
    resolver
        .download_dir()
        .or_else(|_| resolver.document_dir())
        .or_else(|_| resolver.app_local_data_dir())
        .map_err(|_| CommandError::new("document_save_failed"))
}

/// The name arrives from the webview, so it is treated as hostile: separators,
/// parent references and control characters are dropped rather than escaped.
fn safe_filename(filename: &str) -> String {
    let cleaned: String = filename
        .chars()
        .map(|character| {
            if character.is_control() || SEPARATORS.contains(character) {
                '-'
            } else {
                character
            }
        })
        .collect();
    let cleaned = cleaned.trim().trim_matches('.').trim();
    let stem = cleaned
        .strip_suffix(".pdf")
        .or_else(|| cleaned.strip_suffix(".PDF"))
        .unwrap_or(cleaned)
        .trim();
    let stem: String = stem.chars().take(MAX_STEM_CHARS).collect();
    let stem = stem.trim();
    if stem.is_empty() {
        "document.pdf".to_owned()
    } else {
        format!("{stem}.pdf")
    }
}

/// Never overwrite a document already sitting in the folder: a second download
/// of the same bulletin becomes `bulletin (2).pdf`, the way a browser does it.
fn free_path(directory: &Path, filename: &str) -> PathBuf {
    let candidate = directory.join(filename);
    if !candidate.exists() {
        return candidate;
    }
    let stem = filename.strip_suffix(".pdf").unwrap_or(filename);
    for attempt in 2..=MAX_NAME_ATTEMPTS {
        let candidate = directory.join(format!("{stem} ({attempt}).pdf"));
        if !candidate.exists() {
            return candidate;
        }
    }
    directory.join(filename)
}

#[cfg(test)]
mod tests {
    use super::{free_path, safe_filename};

    #[test]
    fn filenames_from_the_webview_cannot_escape_the_download_folder() {
        assert_eq!(safe_filename("../../etc/passwd"), "-..-etc-passwd.pdf");
        assert_eq!(safe_filename(r"C:\Windows\system.ini"), "C--Windows-system.ini.pdf");
        assert_eq!(safe_filename("   "), "document.pdf");
        assert_eq!(safe_filename("bulletin.pdf"), "bulletin.pdf");
        assert_eq!(safe_filename("bulletin"), "bulletin.pdf");
        assert!(safe_filename(&"a".repeat(500)).len() <= 124);
    }

    #[test]
    fn an_existing_document_is_not_overwritten() {
        let directory = std::env::temp_dir().join(format!("betteraimaira-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();

        assert_eq!(free_path(&directory, "bulletin.pdf"), directory.join("bulletin.pdf"));
        std::fs::write(directory.join("bulletin.pdf"), b"%PDF-").unwrap();
        assert_eq!(free_path(&directory, "bulletin.pdf"), directory.join("bulletin (2).pdf"));

        std::fs::remove_dir_all(&directory).unwrap();
    }
}
