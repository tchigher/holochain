#![allow(clippy::mutex_atomic)]
use super::*;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, shrinkwraprs::Shrinkwrap)]
pub struct CallZomeWorkspaceLock(Arc<RwLock<CallZomeWorkspace>>);

impl CallZomeWorkspaceLock {
    pub fn new(workspace: CallZomeWorkspace) -> Self {
        Self(Arc::new(RwLock::new(workspace)))
    }
}

impl From<CallZomeWorkspace> for CallZomeWorkspaceLock {
    fn from(w: CallZomeWorkspace) -> Self {
        Self::new(w)
    }
}
