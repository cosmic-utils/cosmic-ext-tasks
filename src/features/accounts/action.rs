use accounts::{Account, AuthRequest, Provider};
use uuid::Uuid;

use crate::shared::providers::sync::SyncReport;

#[derive(Clone, Debug)]
pub enum AccountsAction {
    Discover,
    Discovered(Result<Vec<Account>, String>),
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
    SetLocallyEnabled(Uuid, bool),
    SyncNow,
    SyncFinished(SyncReport),
    SyncAccount(Uuid),
    SyncAccountFinished(Uuid, SyncReport),
}
