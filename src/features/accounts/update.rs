use std::collections::BTreeSet;

use accounts::Account;
use cosmic::app;

use crate::{app::Message, shared::navigation::core::AppModel};

use super::{manager, AccountsAction};

fn rediscover() -> app::Task<Message> {
    cosmic::task::message(Message::Accounts(AccountsAction::Discover))
}

fn error_text(error: accounts::Error) -> String {
    error.to_string()
}

impl AppModel {
    pub fn update_accounts(&mut self, action: AccountsAction) -> app::Task<Message> {
        match action {
            AccountsAction::Discover => {
                return cosmic::task::future(async move {
                    let result = manager::list().await.map_err(error_text);
                    Message::Accounts(AccountsAction::Discovered(result))
                });
            }
            AccountsAction::Discovered(result) => {
                self.accounts_checked = true;
                let accounts = match result {
                    Ok(accounts) => accounts,
                    Err(error) => {
                        tracing::error!("failed to open the account store: {error}");
                        self.remote_accounts.clear();
                        return self.toast(error);
                    }
                };

                let previously_active = active_ids(&self.remote_accounts);
                let is_first_load = !self.accounts_loaded_once;
                self.accounts_loaded_once = true;
                self.remote_accounts = accounts.clone();

                let active = active_ids(&accounts);

                if !is_first_load {
                    let newly_active: Vec<uuid::Uuid> =
                        active.difference(&previously_active).copied().collect();
                    if !newly_active.is_empty() {
                        let mut disabled = self.config.disabled_accounts.clone();
                        let mut changed = false;
                        for id in newly_active {
                            changed |= disabled.remove(&id);
                        }
                        if changed {
                            if let Err(error) =
                                self.config.set_disabled_accounts(&self.handler, disabled)
                            {
                                tracing::error!("Failed to save account sync preference: {error}");
                            }
                        }
                    }
                }

                let store = self.store.clone();
                let tasks: Vec<app::Task<Message>> = vec![
                    cosmic::task::future(async move {
                        crate::shared::providers::sync::purge_orphaned_accounts(&store, &active);
                        Message::Tasks(crate::shared::navigation::nav::TasksAction::SyncFromDisk)
                    }),
                    cosmic::task::message(Message::Accounts(AccountsAction::SyncNow)),
                ];
                return app::Task::batch(tasks);
            }
            AccountsAction::Changed => return rediscover(),
            AccountsAction::OpenAccounts => {
                self.account_adding = true;
                return app::Task::none();
            }
            AccountsAction::CloseAccounts => {
                self.account_adding = false;
                return app::Task::none();
            }
            AccountsAction::AddProvider(provider) => {
                return cosmic::task::future(async move {
                    let result = manager::begin(provider).await.map_err(error_text);
                    Message::Accounts(AccountsAction::AuthStarted(result))
                });
            }
            AccountsAction::OpenBrowser => {
                if let Some(request) = self.account_auth.clone() {
                    if let Err(error) = accounts::ui::open_sign_in_url(&request) {
                        return self.toast(error.to_string());
                    }
                }
                return app::Task::none();
            }
            AccountsAction::AuthStarted(result) => match result {
                Ok(request) => {
                    self.account_auth = Some(request.clone());
                    self.account_adding = false;
                    let completion = request.clone();
                    let mut tasks = vec![cosmic::task::future(async move {
                        let result = manager::complete(completion).await.map_err(error_text);
                        Message::Accounts(AccountsAction::AuthCompleted(result))
                    })];
                    if let Err(error) = accounts::ui::open_sign_in_url(&request) {
                        tasks.push(self.toast(error.to_string()));
                    }
                    return app::Task::batch(tasks);
                }
                Err(error) => return self.toast(error),
            },
            AccountsAction::AuthCompleted(result) => {
                self.account_auth = None;
                self.account_adding = false;
                match result {
                    Ok(_) => return rediscover(),
                    Err(error) => return self.toast(error),
                }
            }
            AccountsAction::CancelAuth(state) => {
                manager::cancel(&state);
                self.account_adding = false;
                if self
                    .account_auth
                    .as_ref()
                    .is_some_and(|request| request.state == state)
                {
                    self.account_auth = None;
                }
            }
            AccountsAction::Reconnect(id) => {
                return cosmic::task::future(async move {
                    let result = manager::reconnect(id).await.map_err(error_text);
                    Message::Accounts(AccountsAction::AuthStarted(result))
                });
            }
            AccountsAction::Remove(id) => {
                return cosmic::task::future(async move {
                    let result = manager::remove(id).await.map_err(error_text);
                    Message::Accounts(AccountsAction::Removed(result.map(|()| id)))
                });
            }
            AccountsAction::Removed(result) => match result {
                Ok(id) => {
                    self.remote_accounts.retain(|account| account.id != id);
                    return rediscover();
                }
                Err(error) => return self.toast(error),
            },
            AccountsAction::SetLocallyEnabled(id, enabled) => {
                let mut disabled = self.config.disabled_accounts.clone();
                if enabled {
                    disabled.remove(&id);
                } else {
                    disabled.insert(id);
                }
                if let Err(error) = self.config.set_disabled_accounts(&self.handler, disabled) {
                    tracing::error!("Failed to save account sync preference: {error}");
                }
            }
            AccountsAction::SyncFinished(report) => {
                for error in &report.errors {
                    tracing::error!("sync: {error}");
                }
                let needs_reconnect = report
                    .errors
                    .iter()
                    .any(|error| error.to_lowercase().contains("reconnect"));
                let mut tasks = vec![cosmic::task::message(Message::Tasks(
                    crate::shared::navigation::nav::TasksAction::SyncFromDisk,
                ))];
                if needs_reconnect {
                    tasks.push(self.toast(crate::fl!("reconnect-account")));
                }
                return app::Task::batch(tasks);
            }
            AccountsAction::SyncNow => {
                let store = self.store.clone();
                let disabled = self.config.disabled_accounts.clone();
                return cosmic::task::future(async move {
                    let report = crate::shared::providers::sync::run_sync(store, &disabled).await;
                    Message::Accounts(AccountsAction::SyncFinished(report))
                });
            }
            AccountsAction::SyncAccount(id) => {
                if !self.account_syncing.insert(id) {
                    return app::Task::none();
                }
                let store = self.store.clone();
                let disabled = self.config.disabled_accounts.clone();
                return cosmic::task::future(async move {
                    let report =
                        crate::shared::providers::sync::run_sync_one(store, id, &disabled).await;
                    Message::Accounts(AccountsAction::SyncAccountFinished(id, report))
                });
            }
            AccountsAction::SyncAccountFinished(id, report) => {
                self.account_syncing.remove(&id);
                for error in &report.errors {
                    tracing::error!("sync: {error}");
                }
                let mut tasks = vec![cosmic::task::message(Message::Tasks(
                    crate::shared::navigation::nav::TasksAction::SyncFromDisk,
                ))];
                if report.had_errors() {
                    tasks.push(
                        self.toast(crate::fl!("sync-failed", error = report.errors.join("; "))),
                    );
                }
                return app::Task::batch(tasks);
            }
        }

        app::Task::none()
    }
}

fn active_ids(accounts: &[Account]) -> BTreeSet<uuid::Uuid> {
    accounts
        .iter()
        .filter(|account| account.enabled)
        .map(|account| account.id)
        .collect()
}
