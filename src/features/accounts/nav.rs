use accounts::Account;
use uuid::Uuid;

pub struct AccountHeaderMarker(pub Uuid);

/// Email address for a group header.
pub fn account_label(accounts: &[Account], id: Uuid) -> String {
    accounts
        .iter()
        .find(|account| account.id == id)
        .and_then(|account| account.email.as_deref())
        .filter(|email| !email.trim().is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| crate::fl!("account"))
}
