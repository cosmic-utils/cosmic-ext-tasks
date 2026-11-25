// SPDX-License-Identifier: GPL-3.0-only

use accounts::{
    ui::{provider_label, AccountRow, AccountStatus, Labels, View as AccountsView},
    AuthRequest, Provider, Service,
};
use cosmic::Element;

use crate::{
    app::{AppModel, Message},
    fl,
};

use super::AccountsAction;

impl AppModel {
    pub(crate) fn accounts_checked(&self) -> bool {
        self.accounts_checked
    }

    pub(crate) fn account_auth(&self) -> Option<&AuthRequest> {
        self.account_auth.as_ref()
    }

    fn account_locally_enabled(&self, id: uuid::Uuid) -> bool {
        !self.config.disabled_accounts.contains(&id)
    }

    pub(crate) fn is_account_syncing(&self, id: uuid::Uuid) -> bool {
        self.account_syncing.contains(&id)
    }
}

/// Accounts page rendered inside Tasks' dedicated context drawer.
pub fn page(app: &AppModel) -> Element<'_, Message> {
    let rows: Vec<AccountRow<'_>> = app
        .remote_accounts
        .iter()
        .map(|account| AccountRow {
            id: account.id,
            provider: account.provider,
            label: account.email_or_identity().to_owned().into(),
            detail: provider_label(account.provider).into(),
            enabled: app.account_locally_enabled(account.id),
            status: if app.is_account_syncing(account.id) {
                Some(AccountStatus {
                    icon: "dialog-information-symbolic",
                    text: fl!("syncing").into(),
                })
            } else {
                None
            },
        })
        .collect();

    let labels = Labels {
        title: fl!("accounts").into(),
        checking: fl!("accounts-checking").into(),
        empty: fl!("no-accounts-connected").into(),
        no_providers: fl!("accounts-no-providers").into(),
        add: fl!("accounts-add").into(),
        add_title: fl!("accounts-add-title").into(),
        add_body: fl!("accounts-add-body").into(),
        back: fl!("accounts-cancel").into(),
        finish_title: fl!("accounts-finish-title").into(),
        finish_body: &finish_body,
        open_browser_again: fl!("accounts-open-browser").into(),
        cancel: fl!("accounts-cancel").into(),
        reconnect: fl!("accounts-reconnect").into(),
        remove: fl!("accounts-remove").into(),
        sync: fl!("sync").into(),
        sign_in: &sign_in_label,
        ..Labels::default()
    };

    accounts::ui::view(AccountsView {
        service: Service::Tasks,
        accounts: rows,
        pending: app.account_auth(),
        loading: !app.accounts_checked(),
        adding: app.account_adding,
        providers: None,
        form: None,
        footer: None,
        labels,
        on_add_open: &open_accounts,
        on_add_close: &close_accounts,
        on_open_url: &open_browser,
        on_add: &add_provider,
        on_cancel: &cancel_auth,
        on_remove: &remove,
        on_reconnect: &reconnect,
        on_toggle: &set_locally_enabled,
        on_sync: Some(&sync_account),
    })
}

fn finish_body(provider: Provider) -> String {
    fl!("accounts-finish-body", provider = provider_label(provider))
}

fn sign_in_label(provider: Provider) -> String {
    fl!("accounts-sign-in", provider = provider_label(provider))
}

fn open_accounts() -> Message {
    Message::Accounts(AccountsAction::OpenAccounts)
}

fn close_accounts() -> Message {
    Message::Accounts(AccountsAction::CloseAccounts)
}

fn open_browser() -> Message {
    Message::Accounts(AccountsAction::OpenBrowser)
}

fn add_provider(provider: Provider) -> Message {
    Message::Accounts(AccountsAction::AddProvider(provider))
}

fn cancel_auth(state: &str) -> Message {
    Message::Accounts(AccountsAction::CancelAuth(state.to_owned()))
}

fn remove(id: uuid::Uuid) -> Message {
    Message::Accounts(AccountsAction::Remove(id))
}

fn reconnect(id: uuid::Uuid) -> Message {
    Message::Accounts(AccountsAction::Reconnect(id))
}

fn set_locally_enabled(id: uuid::Uuid, enabled: bool) -> Message {
    Message::Accounts(AccountsAction::SetLocallyEnabled(id, enabled))
}

fn sync_account(id: uuid::Uuid) -> Message {
    Message::Accounts(AccountsAction::SyncAccount(id))
}
