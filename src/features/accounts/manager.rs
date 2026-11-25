// SPDX-License-Identifier: GPL-3.0-only

use accounts::{Account, Accounts, AuthRequest, ClientConfig, Config, Provider, SecretString, Service};
use tokio::sync::OnceCell;

const APP_ID: &str = "dev.edfloreshz.Tasks";
static MANAGER: OnceCell<Accounts> = OnceCell::const_new();

pub(crate) async fn manager() -> accounts::Result<&'static Accounts> {
    MANAGER
        .get_or_try_init(|| async {
            // Public OAuth settings for these installed desktop clients.
            let config = Config::new(APP_ID)
                .with_client(
                    Provider::Google,
                    Service::Tasks,
                    ClientConfig {
                        client_id: "111893718611-nsveq8lmkor212ugtt22rk9blpelodgp.apps.googleusercontent.com".into(),
                        client_secret: Some(SecretString::new("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")),
                        redirect_uri: None,
                    },
                )
                .with_client(
                    Provider::Microsoft,
                    Service::Tasks,
                    ClientConfig {
                        client_id: "e43c255a-a104-4931-b24f-2b951221c373".into(),
                        client_secret: None,
                        redirect_uri: None,
                    },
                )
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
    accounts::ui::begin_sign_in(manager().await?, provider, Service::Tasks).await
}

pub(crate) async fn complete(request: AuthRequest) -> accounts::Result<Account> {
    manager().await?.complete_auth(request).await
}

pub(crate) async fn reconnect(id: uuid::Uuid) -> accounts::Result<AuthRequest> {
    manager().await?.reconnect(id).await
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
