use accounts::Account;
use cosmic::widget;
use uuid::Uuid;

pub struct AccountHeaderMarker;

/// Account holder's name for a group header.
pub fn account_label(accounts: &[Account], id: Uuid) -> String {
    accounts
        .iter()
        .find(|account| account.id == id)
        .map(|account| account.display_name.clone())
        .unwrap_or_else(|| crate::fl!("account"))
}

/// Provider icon for an account's nav bar group header.
pub fn provider_icon(provider_id: &str) -> widget::Icon {
    let name = match provider_id {
        "google" => "web-browser-symbolic",
        _ => "weather-clouds-symbolic",
    };
    widget::icon::from_name(name).icon()
}
