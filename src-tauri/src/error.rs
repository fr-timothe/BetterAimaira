use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    pub code: &'static str,
}

impl CommandError {
    pub const fn new(code: &'static str) -> Self {
        Self { code }
    }
}

impl From<reqwest::Error> for CommandError {
    fn from(_: reqwest::Error) -> Self {
        Self::new("portal_unreachable")
    }
}
