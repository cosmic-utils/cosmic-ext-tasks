// SPDX-License-Identifier: GPL-3.0-only

use accounts::{provider::providers_for, ui::provider_label, AuthRequest, Service};
use cosmic::iced::Alignment;
use cosmic::{widget, Element};

use crate::{
    app::{AppModel, Message},
    fl,
};

use super::AccountsAction;

impl AppModel {
    pub(crate) fn accounts_checked(&self) -> bool {
        self.accounts_checked
    }

    pub(crate) fn accounts_error(&self) -> Option<&str> {
        self.accounts_error.as_deref()
    }

    pub(crate) fn account_action_error(&self) -> Option<&str> {
        self.account_action_error.as_deref()
    }

    pub(crate) fn account_auth(&self) -> Option<&AuthRequest> {
        self.account_auth.as_ref()
    }

    fn account_locally_enabled(&self, id: uuid::Uuid) -> bool {
        !self.config.disabled_accounts.contains(&id)
    }
}

/// Accounts section rendered inside Tasks' settings context drawer.
///
/// Accounts are created, reconnected, and removed here. Only accounts whose
/// service is Tasks are listed, even if the library was built with other
/// service features.
pub fn page(app: &AppModel) -> Element<'_, Message> {
    let mut section = widget::settings::section().title(fl!("accounts"));

    if !app.accounts_checked() {
        section = section.add(widget::text::body(fl!("accounts-checking")));
    } else if let Some(error) = app.accounts_error() {
        section = section.add(widget::text::body(error));
    } else if app.remote_accounts.is_empty() {
        section = section.add(widget::text::body(fl!("no-accounts-connected")));
    }

    if let Some(error) = app.account_action_error() {
        section = section.add(widget::text::body(fl!(
            "account-action-error",
            error = error
        )));
    }

    if let Some(request) = app.account_auth() {
        let state = request.state.clone();
        section = section.add(
            widget::settings::item::builder(fl!(
                "accounts-waiting",
                provider = provider_label(request.provider)
            ))
            .description(fl!("accounts-waiting-description"))
            .control(
                widget::button::standard(fl!("accounts-cancel"))
                    .on_press(Message::Accounts(AccountsAction::CancelAuth(state))),
            ),
        );
    }

    for account in &app.remote_accounts {
        let id = account.id;
        let locally_enabled = app.account_locally_enabled(id);
        let controls = widget::row::with_capacity(3)
            .spacing(cosmic::theme::spacing().space_xs)
            .align_y(Alignment::Center)
            .push(
                widget::button::standard(fl!("accounts-reconnect"))
                    .on_press(Message::Accounts(AccountsAction::Reconnect(id))),
            )
            .push(
                widget::button::destructive(fl!("accounts-remove"))
                    .on_press(Message::Accounts(AccountsAction::Remove(id))),
            )
            .push(widget::toggler(locally_enabled).on_toggle(move |enabled| {
                Message::Accounts(AccountsAction::SetLocallyEnabled(id, enabled))
            }));

        section = section.add(
            widget::settings::item::builder(account.email_or_identity().to_owned())
                .description(provider_label(account.provider))
                .control(controls),
        );
    }

    let (label, syncing) = match &app.sync_status {
        Some(crate::shared::providers::sync::SyncStatus::Syncing) => (fl!("syncing"), true),
        Some(crate::shared::providers::sync::SyncStatus::Idle { at, had_errors }) => {
            let base = if *had_errors {
                fl!("sync-error")
            } else {
                fl!("last-synced")
            };
            (
                format!(
                    "{base} ({})",
                    crate::features::tasks::task::Task::format_timestamp(at)
                ),
                false,
            )
        }
        None => (String::new(), false),
    };
    section = section.add(widget::settings::item::item(
        label,
        widget::button::standard(fl!("sync")).on_press_maybe(
            (!syncing && !app.remote_accounts.is_empty())
                .then_some(Message::Accounts(AccountsAction::SyncNow)),
        ),
    ));

    let available = providers_for(Service::Tasks);
    let has_providers = !available.is_empty();
    let mut providers = widget::row::with_capacity(2).spacing(cosmic::theme::spacing().space_xs);
    for provider in available {
        let label = fl!("accounts-sign-in", provider = provider_label(provider));
        providers = providers.push(
            widget::button::standard(label)
                .on_press(Message::Accounts(AccountsAction::AddProvider(provider))),
        );
    }
    if !has_providers {
        section = section.add(widget::text::body(fl!("accounts-no-providers")));
    } else {
        section = section.add(
            widget::settings::item::builder(fl!("accounts-add"))
                .description(fl!("manage-accounts-description"))
                .control(providers),
        );
    }

    section.into()
}
