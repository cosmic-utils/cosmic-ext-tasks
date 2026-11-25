use accounts::{Account, AuthRequest, Provider};
use uuid::Uuid;

use crate::shared::providers::sync::SyncReport;

#[derive(Clone, Debug)]
pub enum AccountsAction {
    /// Load the app-owned Tasks accounts from the embedded manager.
    Discover,
    Discovered(Result<Vec<Account>, String>),
    /// The embedded manager added, removed, reconnected, or toggled an account.
    Changed,
    AddProvider(Provider),
    AuthStarted(Result<AuthRequest, String>),
    AuthCompleted(Result<Account, String>),
    CancelAuth(String),
    Reconnect(Uuid),
    Remove(Uuid),
    Removed(Result<Uuid, String>),
    /// Pauses or resumes sync for one account inside Tasks only.
    SetLocallyEnabled(Uuid, bool),
    SyncNow,
    SyncFinished(SyncReport),
}
