use crate::{domain::models::sync_data_task::CreateSyncDataTaskRequest, errors::Error};
use error_stack::Result;
use std::future::Future;

pub trait VulnService: Clone + Send + Sync + 'static {
    fn create_sync_data_task(
        &self,
        req: CreateSyncDataTaskRequest,
    ) -> impl Future<Output = Result<i64, Error>> + Send;
}

pub trait VulnRepository: Clone + Send + Sync + 'static {
    fn create_sync_data_task(
        &self,
        req: CreateSyncDataTaskRequest,
    ) -> impl Future<Output = Result<i64, Error>> + Send;
}
