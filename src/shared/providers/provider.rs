use super::model::{RemoteList, RemoteTask, RemoteTaskDraft};

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("network error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("{provider} API error ({status}): {message}")]
    Api {
        provider: &'static str,
        status: u16,
        message: String,
    },
    /// The access token was rejected (expired/revoked); the caller should
    /// force a credential refresh and retry once before surfacing this.
    #[error("authorization expired")]
    Unauthorized,
}

pub type ProviderResult<T> = Result<T, ProviderError>;

/// Provider-agnostic remote task backend.
#[allow(dead_code)]
#[async_trait::async_trait]
pub trait RemoteTaskProvider: Send + Sync {
    fn name(&self) -> &'static str;

    async fn create_list(&self, token: &str, name: &str) -> ProviderResult<RemoteList>;
    async fn delete_list(&self, token: &str, remote_id: &str) -> ProviderResult<()>;
    async fn list_lists(&self, token: &str) -> ProviderResult<Vec<RemoteList>>;
    async fn update_list(&self, token: &str, remote_id: &str, name: &str) -> ProviderResult<()>;

    async fn create_task(
        &self,
        token: &str,
        list_remote_id: &str,
        task: &RemoteTaskDraft,
    ) -> ProviderResult<RemoteTask>;
    async fn delete_task(
        &self,
        token: &str,
        list_remote_id: &str,
        remote_id: &str,
    ) -> ProviderResult<()>;
    async fn list_tasks(
        &self,
        token: &str,
        list_remote_id: &str,
    ) -> ProviderResult<Vec<RemoteTask>>;
    async fn update_task(
        &self,
        token: &str,
        list_remote_id: &str,
        remote_id: &str,
        task: &RemoteTaskDraft,
    ) -> ProviderResult<()>;
}
