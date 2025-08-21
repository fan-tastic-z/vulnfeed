use crate::{
    domain::{
        models::sync_data_task::CreateSyncDataTaskRequest,
        ports::{VulnRepository, VulnService},
    },
    errors::Error,
};
use error_stack::Result;

#[derive(Debug, Clone)]
pub struct Service<R>
where
    R: VulnRepository,
{
    repo: R,
}

impl<R> Service<R>
where
    R: VulnRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

impl<R> VulnService for Service<R>
where
    R: VulnRepository,
{
    async fn create_sync_data_task(&self, req: CreateSyncDataTaskRequest) -> Result<i64, Error> {
        let ret = self.repo.create_sync_data_task(req).await?;
        Ok(ret)
    }
}
