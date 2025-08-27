use crate::{
    domain::models::{
        admin_user::AdminUser,
        auth::LoginRequest,
        sync_data_task::{CreateSyncDataTaskRequest, SyncDataTask},
        vuln_information::{
            GetVulnInformationRequest, ListVulnInformationRequest, ListVulnInformationResponseData,
            VulnInformation,
        },
    },
    errors::Error,
};
use error_stack::Result;
use std::future::Future;

pub trait VulnService: Clone + Send + Sync + 'static {
    fn login(&self, req: &LoginRequest) -> impl Future<Output = Result<AdminUser, Error>> + Send;
    fn create_sync_data_task(
        &self,
        req: CreateSyncDataTaskRequest,
    ) -> impl Future<Output = Result<i64, Error>> + Send;
    fn get_sync_data_task(
        &self,
    ) -> impl Future<Output = Result<Option<SyncDataTask>, Error>> + Send;

    fn list_vulnfusion_information(
        &self,
        req: ListVulnInformationRequest,
    ) -> impl Future<Output = Result<ListVulnInformationResponseData, Error>> + Send;

    fn get_vuln_information(
        &self,
        req: GetVulnInformationRequest,
    ) -> impl Future<Output = Result<Option<VulnInformation>, Error>> + Send;
}

pub trait VulnRepository: Clone + Send + Sync + 'static {
    fn login(&self, req: &LoginRequest) -> impl Future<Output = Result<AdminUser, Error>> + Send;
    fn create_sync_data_task(
        &self,
        req: CreateSyncDataTaskRequest,
    ) -> impl Future<Output = Result<i64, Error>> + Send;

    fn get_sync_data_task(
        &self,
    ) -> impl Future<Output = Result<Option<SyncDataTask>, Error>> + Send;

    fn list_vuln_information(
        &self,
        req: ListVulnInformationRequest,
    ) -> impl Future<Output = Result<ListVulnInformationResponseData, Error>> + Send;

    fn get_vuln_information(
        &self,
        req: GetVulnInformationRequest,
    ) -> impl Future<Output = Result<Option<VulnInformation>, Error>> + Send;
}
