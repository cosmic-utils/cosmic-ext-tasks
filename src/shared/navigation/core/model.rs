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
    pub(crate) account_header_entities: HashMap<Uuid, nav_bar::Id>,
    pub(crate) accounts_checked: bool,
    pub(crate) accounts_loaded_once: bool,
    pub(crate) remote_accounts: Vec<Account>,
    pub(crate) account_auth: Option<AuthRequest>,
    pub(crate) account_adding: bool,
    pub(crate) account_syncing: HashSet<Uuid>,
}
