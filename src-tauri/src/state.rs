use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use reqwest::{Client, Url};

use crate::aimaira::{PlanningSettings, PortalPage, PortalResource};

pub struct PortalCacheEntry {
    pub page: PortalPage,
    pub expires_at: Instant,
}

pub struct AimairaSession {
    pub id: u64,
    pub client: Client,
    pub portal_url: Url,
    pub username: String,
    pub planning: PlanningSettings,
    pub portal_cache: HashMap<PortalResource, PortalCacheEntry>,
    pub portal_cache_versions: HashMap<PortalResource, u64>,
}

#[derive(Default)]
pub struct SessionState(pub Mutex<Option<AimairaSession>>);

impl SessionState {
    pub fn with_session<R>(
        &self,
        f: impl FnOnce(&AimairaSession) -> R,
    ) -> Result<R, crate::error::CommandError> {
        let guard = self
            .0
            .lock()
            .map_err(|_| crate::error::CommandError::new("internal_error"))?;
        let session = guard
            .as_ref()
            .ok_or_else(|| crate::error::CommandError::new("session_expired"))?;
        Ok(f(session))
    }
}
