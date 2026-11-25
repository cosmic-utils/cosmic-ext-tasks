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
    OpenAccounts,
    CloseAccounts,
    AddProvider(Provider),
    OpenBrowser,
    AuthStarted(Result<AuthRequest, String>),
    AuthCompleted(Result<Account, String>),
    CancelAuth(String),
    Reconnect(Uuid),
    Remove(Uuid),
    Removed(Result<Uuid, String>),
    /// Pauses or resumes sync for one account inside Tasks only.
    SetLocallyEnabled(Uuid, bool),
    /// Background sync for every active account. The Accounts pane no longer
    /// exposes this as a global button.
    SyncNow,
    SyncFinished(SyncReport),
    /// User-requested sync for one account from its card.
    SyncAccount(Uuid),
    SyncAccountFinished(Uuid, SyncReport),
}
