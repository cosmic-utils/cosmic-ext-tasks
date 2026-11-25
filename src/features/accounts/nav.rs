use accounts::Account;
use cosmic::widget;
use uuid::Uuid;

/// Tags a non-selectable nav bar entry used purely as a section label
/// grouping the lists belonging to one remote account.
pub struct AccountHeaderMarker;

/// Account holder's name for a group header, e.g. "Jane Doe". The provider
/// is conveyed by the header's icon instead of repeating it in the text.
/// Falls back to a generic label if the account is no longer known (e.g. a
/// stale group briefly visible mid-removal).
pub fn account_label(accounts: &[Account], id: Uuid) -> String {
    accounts
        .iter()
        .find(|account| account.id == id)
        .map(|account| account.display_name.clone())
        .unwrap_or_else(|| crate::fl!("account"))
}

/// Provider icon for an account's nav bar group header. The bundled library
/// has no D-Bus provider manifests, so this uses a theme icon per provider.
pub fn provider_icon(provider_id: &str) -> widget::Icon {
    let name = match provider_id {
        "google" => "web-browser-symbolic",
        _ => "weather-clouds-symbolic",
    };
    widget::icon::from_name(name).icon()
}
