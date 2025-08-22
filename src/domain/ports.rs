use crate::{
    domain::models::sync_data_task::{CreateSyncDataTaskRequest, SyncDataTask},
    errors::Error,
};
use error_stack::Result;
use std::future::Future;

pub trait VulnService: Clone + Send + Sync + 'static {
    fn create_sync_data_task(
        &self,
        req: CreateSyncDataTaskRequest,
    ) -> impl Future<Output = Result<i64, Error>> + Send;
    fn get_sync_data_task(
        &self,
    ) -> impl Future<Output = Result<Option<SyncDataTask>, Error>> + Send;
}

pub trait VulnRepository: Clone + Send + Sync + 'static {
    fn create_sync_data_task(
        &self,
        req: CreateSyncDataTaskRequest,
    ) -> impl Future<Output = Result<i64, Error>> + Send;

    fn get_sync_data_task(
        &self,
    ) -> impl Future<Output = Result<Option<SyncDataTask>, Error>> + Send;
}
