use std::collections::{BTreeMap, BTreeSet, btree_map::Entry};

use accounts::Account;
use jiff::Timestamp;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::features::accounts::{list as list_accounts, token};
use crate::features::lists::list::List;
use crate::features::tasks::task::Task;
use crate::shared::store::source::TaskSource;
use crate::shared::store::Store;

use super::google::GoogleTasksProvider;
use super::microsoft::MicrosoftTodoProvider;
use super::model::RemoteTaskDraft;
use super::provider::{ProviderError, RemoteTaskProvider};

// Both scheduled and manual syncs read and write the same list registry.
static SYNC_LOCK: Mutex<()> = Mutex::const_new(());

#[derive(Clone, Debug, Default)]
pub struct SyncReport {
    pub accounts_synced: usize,
    pub lists_pulled: usize,
    pub tasks_pulled: usize,
    pub tasks_pushed: usize,
    pub errors: Vec<String>,
}

impl SyncReport {
    pub fn had_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

pub fn purge_orphaned_accounts(store: &Store, active_account_ids: &BTreeSet<Uuid>) -> usize {
    let lists = match store.lists().load_all() {
        Ok(lists) => lists,
        Err(err) => {
            tracing::error!("Failed to load lists for orphan cleanup: {err}");
            return 0;
        }
    };

    let mut purged = 0;
    for list in lists {
        let Some(account_id) = list.source.account_id() else {
            continue;
        };
        if !active_account_ids.contains(&account_id) {
            match store.trash().trash_list(list.id) {
                Ok(()) => purged += 1,
                Err(err) => tracing::error!("Failed to trash orphaned list: {err}"),
            }
        }
    }
    purged
}

/// Runs a full two-way sync pass against every Tasks account in this app's
/// embedded store, excluding any the user has paused within Tasks (`disabled`).
pub async fn run_sync(store: Store, disabled: &BTreeSet<Uuid>) -> SyncReport {
    let mut report = SyncReport::default();

    let accounts = match list_accounts().await {
        Ok(accounts) => accounts,
        Err(err) => {
            report
                .errors
                .push(format!("Failed to open the account store: {err}"));
            return report;
        }
    };

    for account in accounts
        .into_iter()
        .filter(|account| account.enabled && !disabled.contains(&account.id))
    {
        merge_report(&mut report, sync_one(&store, &account).await);
    }

    report
}

/// Runs a two-way sync pass for one account, used by the per-account button in
/// the Accounts pane. Locally paused accounts are left untouched.
pub async fn run_sync_one(store: Store, id: Uuid, disabled: &BTreeSet<Uuid>) -> SyncReport {
    let mut report = SyncReport::default();

    let accounts = match list_accounts().await {
        Ok(accounts) => accounts,
        Err(err) => {
            report
                .errors
                .push(format!("Failed to open the account store: {err}"));
            return report;
        }
    };

    let Some(account) = accounts.into_iter().find(|account| account.id == id) else {
        report.errors.push(format!("Account {id} was not found"));
        return report;
    };
    if !account.enabled || disabled.contains(&id) {
        return report;
    }

    merge_report(&mut report, sync_one(&store, &account).await);
    report
}

fn merge_report(report: &mut SyncReport, next: SyncReport) {
    report.lists_pulled += next.lists_pulled;
    report.tasks_pulled += next.tasks_pulled;
    report.tasks_pushed += next.tasks_pushed;
    report.accounts_synced += next.accounts_synced;
    report.errors.extend(next.errors);
}

async fn sync_one(store: &Store, account: &Account) -> SyncReport {
    let _sync_guard = SYNC_LOCK.lock().await;
    let mut report = SyncReport::default();

    let Some(provider) = provider_for(account.provider.as_str()) else {
        return report;
    };

    let access_token = match token(account.id, false).await {
        Ok(token) => token,
        Err(err) => {
            report.errors.push(format!(
                "{}: failed to get access token: {err}",
                account.email_or_identity()
            ));
            return report;
        }
    };

    match sync_account(store, provider.as_ref(), account, &access_token).await {
        Ok(account_report) => merge_report(&mut report, account_report),
        Err(err) => report
            .errors
            .push(format!("{}: {err}", account.email_or_identity())),
    }

    report
}

fn apply_remote_to_local(
    task_store: &crate::shared::store::store::TaskStore<'_>,
    task_id: Uuid,
    remote: &super::model::RemoteTask,
    now: Timestamp,
) {
    let mut task = match task_store.get(task_id) {
        Ok(task) => task,
        Err(err) => {
            tracing::error!("Failed to load task for sync: {err}");
            return;
        }
    };
    task.title = remote.title.clone();
    task.notes = remote.notes.clone();
    task.due_date = remote.due_date;
    task.completion_date = if remote.completed {
        task.completion_date.or(Some(now))
    } else {
        None
    };
    task.remote_updated_at = remote.updated_at.or(Some(now));
    task.last_synced_at = Some(now);
    task.dirty = false;
    if let Err(err) = task_store.save_synced(&task) {
        tracing::error!("Failed to apply remote task update: {err}");
    }
}

fn make_source(provider_id: &str, account_id: Uuid, remote_id: String) -> TaskSource {
    match provider_id {
        "google" => TaskSource::Google {
            account_id,
            remote_id,
        },
        _ => TaskSource::Microsoft {
            account_id,
            remote_id,
        },
    }
}

fn matches_account(source: &TaskSource, account_id: Uuid) -> bool {
    source.account_id() == Some(account_id)
}

fn provider_for(id: &str) -> Option<Box<dyn RemoteTaskProvider>> {
    match id {
        "google" => Some(Box::new(GoogleTasksProvider::default())),
        "microsoft" => Some(Box::new(MicrosoftTodoProvider::default())),
        _ => None,
    }
}

async fn sync_account(
    store: &Store,
    provider: &dyn RemoteTaskProvider,
    account: &Account,
    token: &str,
) -> Result<SyncReport, ProviderError> {
    let mut report = SyncReport::default();

    let remote_lists = provider.list_lists(token).await?;
    if remote_lists.iter().any(|list| list.remote_id.is_empty()) {
        report.errors.push("Provider returned a list without an ID".into());
        return Ok(report);
    }
    let local_lists = match store.lists().load_all() {
        Ok(lists) => lists,
        Err(err) => {
            report.errors.push(format!("Failed to load lists: {err}"));
            return Ok(report);
        }
    };

    let mut local_by_remote_id = BTreeMap::new();
    for list in &local_lists {
        if !matches_account(&list.source, account.id) {
            continue;
        }
        let Some(remote_id) = list.source.remote_id() else {
            continue;
        };
        match local_by_remote_id.entry(remote_id) {
            Entry::Vacant(entry) => {
                entry.insert(list);
            }
            Entry::Occupied(entry) if entry.get().id != list.id => {
                // Keep the first list active and preserve the duplicate's tasks in Trash.
                if let Err(err) = store.trash().trash_list(list.id) {
                    report
                        .errors
                        .push(format!("Failed to trash duplicate list: {err}"));
                }
            }
            Entry::Occupied(_) => {}
        }
    }

    let mut seen_remote_ids = BTreeSet::new();
    for remote_list in &remote_lists {
        if !seen_remote_ids.insert(remote_list.remote_id.as_str()) {
            continue;
        }

        let list_id = match local_by_remote_id.get(remote_list.remote_id.as_str()) {
            Some(list) => {
                if list.name != remote_list.title {
                    if let Err(err) = store
                        .lists()
                        .update(list.id, |l| l.name = remote_list.title.clone())
                    {
                        tracing::error!("Failed to update list name: {err}");
                    }
                }
                list.id
            }
            None => {
                let mut list = List::new(&remote_list.title);
                list.source = make_source(
                    account.provider.as_str(),
                    account.id,
                    remote_list.remote_id.clone(),
                );
                if let Err(err) = store.lists().save(&list) {
                    report.errors.push(format!("Failed to save list: {err}"));
                    continue;
                }
                report.lists_pulled += 1;
                list.id
            }
        };

        match sync_list_tasks(
            store,
            provider,
            account,
            token,
            list_id,
            &remote_list.remote_id,
        )
        .await
        {
            Ok(list_report) => {
                report.tasks_pulled += list_report.tasks_pulled;
                report.tasks_pushed += list_report.tasks_pushed;
                report.errors.extend(list_report.errors);
            }
            Err(err) => report.errors.push(format!("{}: {err}", remote_list.title)),
        }
    }

    for (remote_id, local_list) in local_by_remote_id {
        if !seen_remote_ids.contains(remote_id) {
            if let Err(err) = store.trash().trash_list(local_list.id) {
                tracing::error!("Failed to trash remotely-deleted list: {err}");
            }
        }
    }

    Ok(report)
}

async fn sync_list_tasks(
    store: &Store,
    provider: &dyn RemoteTaskProvider,
    account: &Account,
    token: &str,
    list_id: Uuid,
    list_remote_id: &str,
) -> Result<SyncReport, ProviderError> {
    let mut report = SyncReport::default();
    let task_store = store.tasks(list_id);

    let remote_tasks = provider.list_tasks(token, list_remote_id).await?;
    let local_tasks = task_store.load_all().unwrap_or_default();

    let now = Timestamp::now();

    for remote in &remote_tasks {
        let local = local_tasks.iter().find(|t| {
            matches_account(&t.source, account.id)
                && t.source.remote_id() == Some(remote.remote_id.as_str())
        });

        match local {
            Some(local) if local.dirty => {
                let draft = RemoteTaskDraft {
                    title: local.title.clone(),
                    notes: local.notes.clone(),
                    due_date: local.due_date,
                    completed: local.is_completed(),
                };
                if let Err(err) = provider
                    .update_task(token, list_remote_id, &remote.remote_id, &draft)
                    .await
                {
                    report
                        .errors
                        .push(format!("Failed to push task '{}': {err}", local.title));
                    continue;
                }
                let mut updated = local.clone();
                updated.dirty = false;
                updated.remote_updated_at = Some(now);
                updated.last_synced_at = Some(now);
                if let Err(err) = task_store.save_synced(&updated) {
                    tracing::error!("Failed to save synced task: {err}");
                }
                report.tasks_pushed += 1;
            }
            Some(local) => {
                let remote_newer = match local.remote_updated_at {
                    Some(local_updated) => remote.updated_at.is_none_or(|r| r > local_updated),
                    None => true,
                };
                if remote_newer {
                    apply_remote_to_local(&task_store, local.id, remote, now);
                    report.tasks_pulled += 1;
                }
            }
            None => {
                let mut task = Task::new(&remote.title);
                task.notes = remote.notes.clone();
                task.due_date = remote.due_date;
                if remote.completed {
                    task.completion_date = Some(now);
                }
                task.source = make_source(
                    account.provider.as_str(),
                    account.id,
                    remote.remote_id.clone(),
                );
                task.remote_updated_at = remote.updated_at.or(Some(now));
                task.last_synced_at = Some(now);
                if let Err(err) = task_store.save_synced(&task) {
                    report
                        .errors
                        .push(format!("Failed to save pulled task: {err}"));
                    continue;
                }
                report.tasks_pulled += 1;
            }
        }
    }

    for local in &local_tasks {
        if !local.source.is_local() {
            continue;
        }
        let draft = RemoteTaskDraft {
            title: local.title.clone(),
            notes: local.notes.clone(),
            due_date: local.due_date,
            completed: local.is_completed(),
        };
        match provider.create_task(token, list_remote_id, &draft).await {
            Ok(created) => {
                let mut updated = local.clone();
                updated.source =
                    make_source(account.provider.as_str(), account.id, created.remote_id);
                updated.dirty = false;
                updated.remote_updated_at = Some(now);
                updated.last_synced_at = Some(now);
                if let Err(err) = task_store.save_synced(&updated) {
                    tracing::error!("Failed to save pushed task: {err}");
                }
                report.tasks_pushed += 1;
            }
            Err(err) => {
                report
                    .errors
                    .push(format!("Failed to push new task '{}': {err}", local.title));
            }
        }
    }

    for local in &local_tasks {
        if !matches_account(&local.source, account.id) || local.source.is_local() {
            continue;
        }
        let still_exists = remote_tasks
            .iter()
            .any(|r| Some(r.remote_id.as_str()) == local.source.remote_id());
        if !still_exists {
            let trashed = crate::features::tasks::task::TrashedTask::new(local.clone(), list_id);
            if store.trash().save(&trashed).is_ok() {
                let _ = task_store.delete(local.id);
            }
        }
    }

    Ok(report)
}
