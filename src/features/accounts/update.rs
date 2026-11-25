use std::collections::BTreeSet;

use accounts::Account;
use cosmic::app;

use crate::{app::Message, fl, shared::navigation::core::AppModel};

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
                    Ok(accounts) => {
                        self.accounts_error = None;
                        accounts
                    }
                    Err(error) => {
                        tracing::error!("failed to open the account store: {error}");
                        self.remote_accounts.clear();
                        self.accounts_error = Some(error);
                        return app::Task::none();
                    }
                };

                let previously_active = active_ids(&self.remote_accounts);
                let is_first_load = !self.accounts_loaded_once;
                self.accounts_loaded_once = true;
                self.remote_accounts = accounts.clone();

                // Accounts that are deleted or disabled in the embedded store
                // lose their mirrored lists, regardless of Tasks' local pause
                // switch. That switch only pauses sync; it never keeps data.
                let active = active_ids(&accounts);

                // An account that just became active starts syncing without
                // also requiring the local toggle. Skipped on the first load
                // so existing local preferences survive app startup.
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
            AccountsAction::AddProvider(provider) => {
                self.account_action_error = None;
                return cosmic::task::future(async move {
                    let result = manager::begin(provider).await.map_err(error_text);
                    Message::Accounts(AccountsAction::AuthStarted(result))
                });
            }
            AccountsAction::AuthStarted(result) => match result {
                Ok(request) => {
                    self.account_auth = Some(request.clone());
                    self.account_action_error = None;
                    return cosmic::task::future(async move {
                        let result = manager::complete(request).await.map_err(error_text);
                        Message::Accounts(AccountsAction::AuthCompleted(result))
                    });
                }
                Err(error) => self.account_action_error = Some(error),
            },
            AccountsAction::AuthCompleted(result) => {
                self.account_auth = None;
                match result {
                    Ok(_) => {
                        self.account_action_error = None;
                        return rediscover();
                    }
                    Err(error) => self.account_action_error = Some(error),
                }
            }
            AccountsAction::CancelAuth(state) => {
                manager::cancel(&state);
                if self
                    .account_auth
                    .as_ref()
                    .is_some_and(|request| request.state == state)
                {
                    self.account_auth = None;
                }
            }
            AccountsAction::Reconnect(id) => {
                self.account_action_error = None;
                return cosmic::task::future(async move {
                    let result = manager::reconnect(id).await.map_err(error_text);
                    Message::Accounts(AccountsAction::AuthStarted(result))
                });
            }
            AccountsAction::Remove(id) => {
                self.account_action_error = None;
                return cosmic::task::future(async move {
                    let result = manager::remove(id).await.map_err(error_text);
                    Message::Accounts(AccountsAction::Removed(result.map(|()| id)))
                });
            }
            AccountsAction::Removed(result) => match result {
                Ok(id) => {
                    self.remote_accounts.retain(|account| account.id != id);
                    self.account_action_error = None;
                    return rediscover();
                }
                Err(error) => self.account_action_error = Some(error),
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
                let needs_reconnect = report
                    .errors
                    .iter()
                    .any(|error| error.to_lowercase().contains("reconnect"));
                for error in &report.errors {
                    tracing::error!("sync: {error}");
                }
                self.sync_status = Some(crate::shared::providers::sync::SyncStatus::Idle {
                    at: jiff::Timestamp::now(),
                    had_errors: report.had_errors(),
                });
                let mut tasks = vec![cosmic::task::message(Message::Tasks(
                    crate::shared::navigation::nav::TasksAction::SyncFromDisk,
                ))];
                if needs_reconnect {
                    tasks.push(
                        self.toasts
                            .push(cosmic::widget::Toast::new(fl!("reconnect-account")))
                            .map(cosmic::Action::App),
                    );
                }
                return app::Task::batch(tasks);
            }
            AccountsAction::SyncNow => {
                self.sync_status = Some(crate::shared::providers::sync::SyncStatus::Syncing);
                let store = self.store.clone();
                let disabled = self.config.disabled_accounts.clone();
                return cosmic::task::future(async move {
                    let report = crate::shared::providers::sync::run_sync(store, &disabled).await;
                    Message::Accounts(AccountsAction::SyncFinished(report))
                });
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
