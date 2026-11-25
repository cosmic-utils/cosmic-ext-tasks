use std::collections::{HashMap, HashSet, VecDeque};

use accounts::{Account, AuthRequest};
use cosmic::{
    app::Core,
    cosmic_config,
    iced::keyboard::Modifiers,
    widget::{about::About, menu::key_bind::KeyBind, nav_bar},
};
use uuid::Uuid;

use crate::{
    config,
    features::{
        favorites::favorites::Favorites, lists::content::Content, search::search::Search,
        tasks::details::Details, trash::trash::Trash,
    },
    shared::{dialogs::DialogPage, navigation::ui::MenuAction, store::Store},
};

pub struct AppModel {
    pub(crate) about: About,
    pub(crate) core: Core,
    pub(crate) config: config::AppConfig,
    pub(crate) content: Content,
    pub(crate) context_page: super::context::ContextPage,
    pub(crate) details: Details,
    pub(crate) dialog_pages: VecDeque<DialogPage>,
    pub(crate) dialog_text_input: cosmic::widget::Id,
    pub(crate) favorites: Favorites,
    pub(crate) favorites_entity: nav_bar::Id,
    pub(crate) handler: cosmic_config::Config,
    pub(crate) key_binds: HashMap<KeyBind, MenuAction>,
    pub(crate) modifiers: Modifiers,
    pub(crate) nav: nav_bar::Model,
    pub(crate) search: Search,
    pub(crate) sent_reminders: HashSet<(Uuid, i64)>,
    pub(crate) store: Store,
    pub(crate) toasts: cosmic::widget::Toasts<super::message::Message>,
    pub(crate) trash: Trash,
    pub(crate) trash_entity: nav_bar::Id,

    /// Account id -> its nav bar group-header entity, reused across
    /// `reposition_special_items()` calls instead of being recreated (flicker).
    pub(crate) account_header_entities: HashMap<Uuid, nav_bar::Id>,

    /// Set once the embedded account store has been opened at least once.
    /// Sign-in stays unavailable until then; a locked or missing secure
    /// store is reported through `accounts_error` instead of falling back.
    pub(crate) accounts_checked: bool,
    pub(crate) accounts_error: Option<String>,

    /// Set once the account list has been loaded at least once, so the
    /// "an account just became active" auto-enable logic doesn't treat
    /// every already-active account as newly active on the very first load
    /// and wipe the user's saved local on/off preferences.
    pub(crate) accounts_loaded_once: bool,
    pub(crate) remote_accounts: Vec<Account>,

    /// An OAuth request that has opened the browser and is waiting to finish.
    pub(crate) account_auth: Option<AuthRequest>,
    pub(crate) account_action_error: Option<String>,

    pub(crate) sync_status: Option<crate::shared::providers::sync::SyncStatus>,
}
