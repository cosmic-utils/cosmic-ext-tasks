// SPDX-License-Identifier: GPL-3.0-only

use accounts::{Account, Accounts, AuthRequest, Config, Provider, Service};
use tokio::sync::OnceCell;

const APP_ID: &str = "dev.edfloreshz.Tasks";
static MANAGER: OnceCell<Accounts> = OnceCell::const_new();

pub(crate) async fn manager() -> accounts::Result<&'static Accounts> {
    MANAGER
        .get_or_try_init(|| async {
            let config = Config::new(APP_ID)
                .with_client_from_env(Provider::Google, Service::Tasks)
                .with_client_from_env(Provider::Microsoft, Service::Tasks);
            Accounts::new(config).await
        })
        .await
}

pub(crate) async fn list() -> accounts::Result<Vec<Account>> {
    Ok(manager().await?.list(Service::Tasks).await)
}

pub(crate) async fn begin(provider: Provider) -> accounts::Result<AuthRequest> {
    let manager = manager().await?;
    accounts::ui::sign_in(manager, provider, Service::Tasks).await
}

pub(crate) async fn complete(request: AuthRequest) -> accounts::Result<Account> {
    manager().await?.complete_auth(request).await
}

pub(crate) async fn reconnect(id: uuid::Uuid) -> accounts::Result<AuthRequest> {
    let request = manager().await?.reconnect(id).await?;
    open::that(&request.url).map_err(|error| {
        accounts::Error::AuthenticationFailed(format!("could not open browser: {error}"))
    })?;
    Ok(request)
}

pub(crate) fn cancel(state: &str) {
    if let Some(manager) = MANAGER.get() {
        let state = state.to_owned();
        tokio::spawn(async move {
            manager.cancel_auth(&state).await;
        });
    }
}

pub(crate) async fn remove(id: uuid::Uuid) -> accounts::Result<()> {
    manager().await?.remove(id).await
}

/// Returns a token, refreshing it when the cached one is missing or stale.
pub(crate) async fn token(id: uuid::Uuid, refresh: bool) -> accounts::Result<String> {
    manager()
        .await?
        .access_token(id, Service::Tasks, refresh)
        .await
}
